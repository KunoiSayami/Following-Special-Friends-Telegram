#!/usr/bin/env python
# -*- coding: utf-8 -*-
# watch.py
# Copyright (C) 2020 KunoiSayami
#
# This module is part of Following-Special-Friends-Telegram and is released under
# the AGPL v3 License: https://www.gnu.org/licenses/agpl-3.0.txt
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with this program. If not, see <https://www.gnu.org/licenses/>.
import ast
import asyncio
import logging
import time
from configparser import ConfigParser
from dataclasses import dataclass

from typing import List

import pyrogram
from pyrogram import Client, filters, ContinuePropagation
from pyrogram.handlers import MessageHandler
from pyrogram.types import Message


@dataclass(init=False)
class LastMessage:
    last_send: float
    client: Client
    group: List[str]
    owner: int
    message: Message

    def __init__(self, owner: int, client: Client):
        self.client = client
        self.last_send = time.time()
        self.owner = owner
        self.group = []
        self.message = None

    async def send_message(self, text: str) -> None:
        if not self.check():
            self.group = [text]
            self.message = await self.client.send_message(self.owner, text, parse_mode='html')
            self.last_send = time.time()
        else:
            self.group.append(text)
            await self.message.edit('\n'.join(text), 'html')

    def check(self) -> bool:
        return time.time() - self.last_send > 60


class Watcher:
    def __init__(self, owner: int, bot_token: str, api_id: int, api_hash: str, watch_list: List[int]):
        self.logger: logging.Logger = logging.getLogger('Watcher')
        self.logger.setLevel(logging.DEBUG)
        self.human: Client = Client('watcher', api_id=api_id, api_hash=api_hash)
        self.bot: Client = Client('watchbot', api_id=api_id, api_hash=api_hash, bot_token=bot_token)
        self.owner: int = owner
        self.watch_list = watch_list
        self.lock: asyncio.Lock = asyncio.Lock()
        self.last_message: LastMessage = LastMessage(owner, self.bot)

    def init_handler(self) -> None:
        self.human.add_handler(MessageHandler(self.watch_media, filters.group
                                              & filters.text & filters.user(self.watch_list)))
        self.human.add_handler(MessageHandler(self.watch_text, filters.group
                                              & filters.text & filters.user(self.watch_list)))

    async def start(self) -> None:
        self.logger.debug('Starting bot account')
        await self.bot.start()
        self.logger.debug('Starting main account')
        await self.human.start()
        # self.owner = (await self.human.get_me()).id
        self.logger.info('Start bot successfully')

    @staticmethod
    async def idle() -> None:
        await pyrogram.idle()

    async def stop(self) -> None:
        self.logger.debug('Stopping...')
        await asyncio.gather(self.human.stop(), self.bot.stop())

    async def watch_text(self, _client: Client, msg: Message) -> None:
        async with self.lock:
            await self.last_message.send_message(self.paste(msg))
        raise ContinuePropagation

    async def watch_media(self, _client: Client, msg: Message) -> None:
        async with self.lock:
            await self.last_message.send_message(self.paste_media(msg))
        raise ContinuePropagation

    @staticmethod
    def paste(msg: Message) -> str:
        return f'<b>{msg.from_user.first_name}</b><a href="http://t.me/{- msg.chat.id + 1000000000000}/' \
               f'{msg.message_id}">: {msg.text[:20]}'

    @staticmethod
    def paste_media(msg: Message) -> str:
        return f'<b>{msg.from_user.first_name}</b><a href="http://t.me/{- msg.chat.id + 1000000000000}/' \
               f'{msg.message_id}">: sent a {Watcher.get_media_type(msg)}'

    @staticmethod
    def get_media_type(msg: Message) -> str:
        return 'photo' if msg.photo else \
            'video' if msg.video else \
            'animation' if msg.animation else \
            'sticker' if msg.sticker else \
            'voice' if msg.voice else \
            'document' if msg.document else \
            'audio' if msg.audio else \
            'contact' if msg.contact else 'error'


async def main():
    config = ConfigParser()
    config.read('config.ini')
    watcher = Watcher(
        config.getint('telegram', 'owner'),
        config.get('telegram', 'bot_token'), config.getint('telegram', 'api_id'),
        config.get('telegram', 'api_hash'), ast.literal_eval(config.get('follow', 'list')))
    await watcher.start()
    await watcher.idle()
    await watcher.stop()


if __name__ == '__main__':
    try:
        import coloredlogs

        coloredlogs.install(logging.DEBUG, fmt='%(asctime)s,%(msecs)03d - %(levelname)s - %(name)s - '
                                               '%(funcName)s - %(lineno)d - %(message)s')
    except ModuleNotFoundError:
        logging.basicConfig(level=logging.DEBUG, format='%(asctime)s - %(levelname)s - '
                                                        '%(name)s - %(funcName)s - %(lineno)d - %(message)s')
    logging.getLogger('pyrogram').setLevel(logging.WARNING)
    asyncio.get_event_loop().run_until_complete(main())
