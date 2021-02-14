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
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn get_current_timestamp() -> u128 {
    let start = std::time::SystemTime::now();
    let since_the_epoch = start
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

struct LastSend {
    timestamp: u128
}

struct SpecialFollowing {
    objects: HashMap<u64, Arc<Mutex<LastSend>>>
}

impl LastSend {

    fn check(&self) -> bool {
        get_current_timestamp() - self.timestamp < 60
    }

    fn update(&mut self) {
        self.timestamp = get_current_timestamp();
    }
}

async fn handle_update(mut client: ClientHandle, updates: UpdateIter) -> Result<()> {
    for update in updates {
        match update {
            Update::NewMessage(message) => {
            }
            _ => {}
        }
    }

    Ok(())
}

async fn async_main(config: configure::configparser::Configure) -> Result<()> {
    let client = functions::telegram::try_connect(
        config.api_id,
        &config.api_hash,
        "data/human.session").await?;
    let mut handle = client.handle();
    let network_handle = task::spawn(async move { client.run_until_disconnected().await });
    handle.disconnect().await;
    network_handle.await??;
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