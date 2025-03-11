/*
 ** Copyright (C) 2021-2025 KunoiSayami
 **
 ** This file is part of Following-Special-Friends-Telegram and is released under
 ** the AGPL v3 License: https://www.gnu.org/licenses/agpl-3.0.txt
 **
 ** This program is free software: you can redistribute it and/or modify
 ** it under the terms of the GNU Affero General Public License as published by
 ** the Free Software Foundation, either version 3 of the License, or
 ** 6any later version.
 **
 ** This program is distributed in the hope that it will be useful,
 ** but WITHOUT ANY WARRANTY; without even the implied warranty of
 ** MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 ** GNU Affero General Public License for more details.
 **
 ** You should have received a copy of the GNU Affero General Public License
 ** along with this program. If not, see <https://www.gnu.org/licenses/>.
 */
mod configure;
mod functions;

use crate::configure::prelude::*;
use clap::arg;
use grammers_client::types::{Chat, Media, Message};
use grammers_client::Update;
use kstool::prelude::get_current_timestamp;
use log::{debug, error, info, warn};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::{runtime, task};

#[derive(Clone, Debug)]
enum MessageCommand {
    Message(i64, u128, String),
    Terminate,
}

fn build_message_string(message: &Message, chat_id: i64, user_id: i64, user_name: &str) -> String {
    // TODO: message type support
    let type_string = if let Some(media) = message.media() {
        match media {
            Media::Photo(_) => "photo",
            Media::Document(_) => "document",
            Media::Sticker(_) => "sticker",
            _ => "unsupported media",
        }
    } else {
        "text"
    };
    let message_id = message.id();
    format!(
        "[{user_name}](tg://user?id={user_id}) send a [{type_string}](https://t.me/c/{chat_id}/{message_id}) message"
    )
}

async fn handle_update(
    update: Update,
    special_list: Arc<HashSet<i64>>,
    sender: mpsc::Sender<MessageCommand>,
) -> anyhow::Result<()> {
    match update {
        Update::NewMessage(message) => match message.chat() {
            Chat::Group(chat) => match message.sender() {
                Some(user) => {
                    let message_sender = user.id();
                    debug!("Get sender id: {message_sender}");
                    if !special_list.contains(&message_sender)
                        || (message.text().is_empty() && message.media().is_none())
                    {
                        return Ok(());
                    }

                    sender
                        .send(MessageCommand::Message(
                            user.id(),
                            get_current_timestamp(),
                            build_message_string(
                                &message,
                                chat.id(),
                                user.id(),
                                user.name().unwrap_or("N\\/A"),
                            ),
                        ))
                        .await
                        .inspect_err(|_| error!("Unable send message to another future"))
                        .ok();
                }
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }

    Ok(())
}

async fn message_handler(
    mut receiver: mpsc::Receiver<MessageCommand>,
    mut last_send: HashMap<i64, u128>,
) -> anyhow::Result<()> {
    let client = reqwest::Client::new();
    while let Some(msg) = receiver.recv().await {
        match msg {
            MessageCommand::Message(sender, message_timestamp, text) => {
                let post_data = functions::telegram::SendMessageParameters::new(
                    *BOT_OWNER.get().unwrap(),
                    text,
                );

                let timestamp = last_send.get_mut(&sender).unwrap();
                debug!("Last send message timestamp: {timestamp}");
                let last_time = get_current_timestamp();
                if last_time - *timestamp > *BOT_DURATION.get().unwrap() * 1000 {
                    client
                        .post(
                            format!(
                                "{}/bot{}/sendMessage",
                                BOT_API_SERVER.get().unwrap(),
                                BOT_TOKEN.get().unwrap(),
                            )
                            .as_str(),
                        )
                        .json(&post_data)
                        .send()
                        .await?;
                    info!("Send message successful");
                    *timestamp = message_timestamp;
                }
            }
            MessageCommand::Terminate => break,
        }
    }
    Ok(())
}

async fn async_main(config: Configure, session_path: String) -> anyhow::Result<()> {
    let client =
        functions::telegram::try_connect(config.api_id(), &config.api_hash(), &session_path)
            .await?;

    let hashset_list = Arc::new(config.following().clone());

    let (sender, receiver) = mpsc::channel(1024);

    let mut tasks = vec![];

    let sender_thread = tokio::spawn(message_handler(receiver, {
        let mut map: HashMap<i64, u128> = Default::default();
        for x in config.following().clone() {
            map.insert(x, 0);
            debug!("Insert {x} to following list");
        }
        map
    }));

    while let Some(updates) = tokio::select! {
        _ = tokio::signal::ctrl_c() => Ok(None),
        result = client.next_update() => result.map(Some),
    }? {
        let l = hashset_list.clone();
        let sender = sender.clone();
        tasks.push(task::spawn(async move {
            match handle_update(updates, l, sender).await {
                Ok(_) => {}
                Err(e) => error!("Error handling updates: {e}"),
            }
        }));
    }
    sender
        .send(MessageCommand::Terminate)
        .await
        .inspect_err(|_| error!("Unable send terminate command"))
        .ok();

    for task in tasks {
        task.await?;
    }
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            warn!("Force exit from waiting sender thread");
            return Ok(())
        },
        ret = sender_thread => {
            ret??;
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let matches = clap::command!()
        .args(&[
            arg!(--config <CONFIG_FILE> "Override default configure file location")
                .default_value("config.toml"),
            arg!(--session <SESSION_FILE> "Override default session file location")
                .default_value("human.session"),
        ])
        .get_matches();

    env_logger::Builder::from_default_env().init();

    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main(
            Configure::new(matches.get_one::<String>("config").unwrap())?,
            matches
                .get_one("session")
                .map(|s: &String| s.to_string())
                .unwrap(),
        ))?;

    Ok(())
}
