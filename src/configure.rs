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
mod configparser {
    use crate::functions::Result;
    use serde_derive::Deserialize;
    use std::collections::HashSet;
    use std::path::Path;

    #[derive(Deserialize)]
    struct Telegram {
        api_id: i32,
        api_hash: String,
        bot_token: String,
        owner: i32,
        api_address: Option<String>,
    }

    #[derive(Deserialize)]
    struct _Following {
        list: Vec<i64>,
        duration: Option<u64>,
    }

    #[derive(Deserialize)]
    struct _Configure {
        telegram: Telegram,
        follow: _Following,
    }

    impl _Configure {
        fn new<P: AsRef<Path>>(path: P) -> Result<_Configure> {
            let contents = std::fs::read_to_string(path)?;
            let contents_str = contents.as_str();
            let configure: _Configure = toml::from_str(contents_str)?;
            Ok(configure)
        }
    }

    pub struct Configure {
        api_id: i32,
        api_hash: String,
        bot_token: String,
        owner: i32,
        following: HashSet<i64>,
        api_address: String,
        duration: u128,
    }

    impl Configure {
        pub fn new<P: AsRef<Path>>(path: P) -> Result<Configure> {
            let _configure = _Configure::new(path)?;
            Ok(Configure {
                api_id: _configure.telegram.api_id,
                api_hash: _configure.telegram.api_hash,
                bot_token: _configure.telegram.bot_token,
                owner: _configure.telegram.owner,
                following: _configure.follow.list.into_iter().collect(),
                api_address: match _configure.telegram.api_address {
                    Some(address) => address,
                    None => String::from("https://api.telegram.org"),
                },
                duration: _configure.follow.duration.unwrap_or(60) as u128,
            })
        }

        pub fn api_id(&self) -> i32 {
            self.api_id
        }
        pub fn api_hash(&self) -> &str {
            &self.api_hash
        }
        pub fn bot_token(&self) -> &str {
            &self.bot_token
        }
        pub fn owner(&self) -> i32 {
            self.owner
        }
        pub fn following(&self) -> &HashSet<i64> {
            &self.following
        }
        pub fn api_address(&self) -> &str {
            &self.api_address
        }
        pub fn duration(&self) -> u128 {
            self.duration
        }
    }
}

pub use configparser::Configure;
