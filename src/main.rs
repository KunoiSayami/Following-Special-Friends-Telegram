/*
 ** Copyright (C) 2021 KunoiSayami
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

use functions::Result;
use grammers_client::types::{Chat, Media, Message};
use grammers_client::Update;
use kstool::prelude::get_current_timestamp;
use log::{debug, error, info};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::{runtime, task};

#[derive(Clone)]
struct BotConfigure {
    basic_api_address: String,
    bot_token: String,
    owner: i32,
}

impl BotConfigure {
    async fn send_message(&self, text: String) -> Result<()> {
        let client = reqwest::Client::new();
        let post_data = functions::telegram::SendMessageParameters::new(self.owner, text);
        client
            .post(
                format!(
                    "{}/bot{}/sendMessage",
                    self.basic_api_address, self.bot_token
                )
                .as_str(),
            )
            .json(&post_data)
            .send()
            .await?;
        Ok(())
    }
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
    //let message_type = if message.media().is_none() {"text"} else {type_string};
    let message_id = message.id();
    format!(
        "[{}](tg://user?id={}) send a [{}](https://t.me/c/{}/{}) message",
        user_name, user_id, type_string, chat_id, message_id
    )
}

async fn handle_update(
    update: Update,
    special_list: HashSet<i64>,
    bot: &BotConfigure,
    lock: &Arc<Mutex<HashMap<i64, u128>>>,
    duration: u128,
) -> Result<()> {
    match update {
        Update::NewMessage(message) => match message.chat() {
            Chat::Group(chat) => match message.sender() {
                Some(user) => {
                    let sender = user.id();
                    info!("Get sender id: {}", sender);
                    if special_list.contains(&sender) {
                        if message.text().is_empty() && message.media().is_none() {
                            return Ok(());
                        }
                        let s = build_message_string(&message, chat.id(), user.id(), user.name());
                        {
                            let mut last_send = lock.lock().await;
                            let timestamp = last_send.get_mut(&sender).unwrap();
                            debug!("Last send message timestamp: {}", *timestamp);
                            let last_time = get_current_timestamp();
                            if last_time - *timestamp > duration * 1000 {
                                info!("Send message successful");
                                bot.send_message(s).await?;
                                *timestamp = get_current_timestamp();
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        },
        _ => {}
    }

    Ok(())
}

async fn async_main(config: configure::Configure) -> Result<()> {
    let client =
        functions::telegram::try_connect(config.api_id(), &config.api_hash(), "data/human.session")
            .await?;

    let last_update = Arc::new(Mutex::new({
        let mut map: HashMap<i64, u128> = Default::default();
        for x in config.following().clone() {
            map.insert(x, 0);
            debug!("Insert {} to following list", x);
        }
        map
    }));
    let hashset_list = config.following().clone();
    let owner = config.owner().to_owned();
    let duration = config.duration().to_owned();

    let bot_configure = BotConfigure {
        bot_token: config.bot_token().to_string(),
        basic_api_address: config.api_address().to_string(),
        owner,
    };

    let mut tasks = vec![];

    while let Some(updates) = tokio::select! {
        _ = tokio::signal::ctrl_c() => Ok(None),
        result = client.next_update() => result,
    }? {
        let special_list = hashset_list.clone();
        let config = bot_configure.clone();
        let last_update_lock = Arc::clone(&last_update);
        tasks.push(task::spawn(async move {
            match handle_update(updates, special_list, &config, &last_update_lock, duration).await {
                Ok(_) => {}
                Err(e) => error!("Error handling updates: {}", e),
            }
        }));
    }

    for task in tasks {
        task.await?;
    }

    Ok(())
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env().init();

    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main(configure::Configure::new("data/config.toml")?))?;

    Ok(())
}
