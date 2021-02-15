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

use grammers_client::{Client, ClientHandle, Config, InitParams, Update, UpdateIter};
use simple_logger::SimpleLogger;
use tokio::{runtime, task};
use functions::Result;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use grammers_client::types::Chat;
use log::{error, info};

fn get_current_timestamp() -> u128 {
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

async fn handle_update(mut client: ClientHandle,
                       updates: UpdateIter,
                       special_list: HashSet<i32>,
                       owner: i32,
                       last_send_lock: Arc<Mutex<HashMap<i32, u128>>>) -> Result<()> {
    for update in updates {
        match update {
            Update::NewMessage(message) => {
                match message.sender() {
                    Some(chat) => {
                        let sender = chat.id();
                        info!("Get sender id: {}", sender);
                        if special_list.contains(&sender) {
                            let mut last_send = last_send_lock.lock().unwrap();
                            // TODO: send message to owner
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}

async fn async_main(config: configure::configparser::Configure) -> Result<()> {
    let mut client = functions::telegram::try_connect(
        config.api_id,
        &config.api_hash,
        "data/human.session").await?;
    let mut bot_client = functions::telegram::try_connect_bot(
        config.api_id,
        &config.api_hash,
        "data/bot.session",
        &config.bot_token
    ).await?;

    let last_update = Arc::new(Mutex::new({
        let mut map: HashMap<i32, u128> = Default::default();
        config.following.clone().iter().map(|x| {
            map.insert(*x, 0);
        });
        map
    }));
    let hashset_list = (&config.following).clone();
    let owner = config.owner.to_owned();

    let mut handle = client.handle();
    //let network_handle = task::spawn(async move { client.run_until_disconnected().await })
    while let Some(updates) = client.next_updates().await? {
        let bot_handle = bot_client.handle();
        let special_list = hashset_list.clone();
        let last_update = last_update.clone();
        task::spawn(async move {
            match handle_update(bot_handle, updates, special_list, owner, last_update).await {
                Ok(_) => {}
                Err(e) => error!("Error handling updates: {}", e)
            }
        });
        bot_client.session().save()?;
    }
    handle.disconnect().await;
    //network_handle.await??;
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