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
mod configparser {
    use once_cell::sync::OnceCell;
    use serde::Deserialize;
    use std::collections::HashSet;
    use std::path::Path;

    pub static BOT_TOKEN: OnceCell<String> = OnceCell::new();
    pub static BOT_API_SERVER: OnceCell<String> = OnceCell::new();
    pub static BOT_OWNER: OnceCell<i64> = OnceCell::new();
    pub static BOT_DURATION: OnceCell<u128> = OnceCell::new();

    #[derive(Deserialize)]
    struct Telegram {
        api_id: i32,
        api_hash: String,
        bot_token: String,
        owner: i64,
        api_address: Option<String>,
    }

    #[derive(Deserialize)]
    struct PrivFollowing {
        list: Vec<i64>,
        duration: Option<u64>,
    }

    #[derive(Deserialize)]
    struct PrivConfigure {
        telegram: Telegram,
        follow: PrivFollowing,
    }

    impl PrivConfigure {
        fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<PrivConfigure> {
            let contents = std::fs::read_to_string(path)?;
            let contents_str = contents.as_str();
            let configure: PrivConfigure = toml::from_str(contents_str)?;
            Ok(configure)
        }
    }

    pub struct Configure {
        api_id: i32,
        api_hash: String,
        following: HashSet<i64>,
    }

    impl Configure {
        pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Configure> {
            let _configure = PrivConfigure::new(path)?;
            BOT_OWNER.set(_configure.telegram.owner).unwrap();
            BOT_API_SERVER
                .set(
                    _configure
                        .telegram
                        .api_address
                        .unwrap_or_else(|| "https://api.telegram.org".to_string()),
                )
                .unwrap();
            BOT_TOKEN.set(_configure.telegram.bot_token).unwrap();
            BOT_DURATION
                .set(_configure.follow.duration.unwrap_or(60) as u128)
                .unwrap();
            Ok(Configure {
                api_id: _configure.telegram.api_id,
                api_hash: _configure.telegram.api_hash,
                following: _configure.follow.list.into_iter().collect(),
            })
        }

        pub fn api_id(&self) -> i32 {
            self.api_id
        }
        pub fn api_hash(&self) -> &str {
            &self.api_hash
        }
        pub fn following(&self) -> &HashSet<i64> {
            &self.following
        }
    }
}

pub mod prelude {
    pub use super::configparser::{Configure, BOT_API_SERVER, BOT_DURATION, BOT_OWNER, BOT_TOKEN};
}
