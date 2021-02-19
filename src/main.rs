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
mod functions;
mod configure;

use grammers_client::{Update, UpdateIter, ClientHandle};
use simple_logger::SimpleLogger;
use tokio::{runtime, task};
use tokio::sync::Mutex;
use tokio::signal::unix::{signal, SignalKind};
use functions::Result;
use std::collections::{HashMap, HashSet};
use grammers_client::types::{Chat, Message};
use log::{error, debug, info};
use std::sync::Arc;

fn get_current_timestamp() -> u128 {
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

#[derive(Clone)]
struct BotConfigure {
    basic_api_address: String,
    bot_token: String,
    owner: i32,
}

impl BotConfigure {
    async fn send_message(&self, text: String) -> Result<()>{
        let client = reqwest::Client::new();
        let post_data = functions::telegram::SendMessageParameters::new(self.owner, text);
        client.post(format!("{}/bot{}/sendMessage", self.basic_api_address, self.bot_token).as_str())
            .json(&post_data)
            .send()
            .await?;
        Ok(())
    }
}


fn build_message_string(message: &Message, chat_id: i32, user_name: &str) -> String {
    // TODO: message type support
    /*let type_string = if let Some(media) = message.media() {

    }*/
    let message_type = if message.media().is_none() {"text"} else {"media"};
    let message_id = message.id();
    format!("{} send a [{}](https://t.me/c/{}/{}) message", user_name, message_type, chat_id, message_id)
}

async fn handle_update(updates: UpdateIter,
                       special_list: HashSet<i32>,
                       bot: &BotConfigure,
                       lock: &Arc<Mutex<HashMap<i32, u128>>>,
                       duration: u128
) -> Result<()> {
    for update in updates {
        match update {
            Update::NewMessage(message) => {
                match message.chat() {
                    Chat::Group(chat) =>
                        match message.sender() {
                            Some(user) => {
                                let sender = user.id();
                                info!("Get sender id: {}", sender);
                                if special_list.contains(&sender) {
                                    if message.text().is_empty() && message.media().is_none() {
                                        continue
                                    }
                                    let s = build_message_string(&message, chat.id(), user.name());
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
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}

async fn ctrl_c_handler(mut handle: ClientHandle) -> Result<()> {

    let mut stream = signal(SignalKind::interrupt())?;
    loop {
        stream.recv().await;
        info!("got SIGINT");
        handle.disconnect().await;
        debug!("Break from ctrl c handler loop");
        break Ok(())
    }
}

async fn async_main(config: configure::configparser::Configure) -> Result<()> {
    let mut client = functions::telegram::try_connect(
        config.api_id,
        &config.api_hash,
        "data/human.session").await?;


    let last_update = Arc::new(Mutex::new({
        let mut map: HashMap<i32, u128> = Default::default();
        for x in config.following.clone() {
            map.insert(x, 0);
            debug!("Insert {} to following list", x);
        }
        map
    }));
    let hashset_list = config.following.clone();
    let owner = config.owner.to_owned();
    let duration = config.duration.to_owned();

    let handle = client.handle();
    let bot_configure = BotConfigure{
        bot_token: config.bot_token.clone(),
        basic_api_address: config.api_address.clone(),
        owner
    };

    let mut tasks = vec![];
    let _signal_task = task::spawn(async move {
        match ctrl_c_handler(handle).await {
            Ok(_) => {}
            Err(e) => error!("Error start signal listener: {}", e)
        }
    });

    while let Some(updates) = client.next_updates().await? {
        let special_list = hashset_list.clone();
        let config = bot_configure.clone();
        let last_update_lock = Arc::clone(&last_update);
        tasks.push(task::spawn(async move {
            match handle_update(updates, special_list, &config, &last_update_lock, duration).await {
                Ok(_) => {}
                Err(e) => error!("Error handling updates: {}", e)
            }
        }));
        client.session().save()?;
    }


    for task in tasks {
        task.await?;
    }

    //signal_task.await?;

    Ok(())
}

fn main() -> Result<()> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();


    runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async_main(configure::configparser::Configure::new("data/config.toml")?))?;

    Ok(())
}