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
pub(crate) mod configparser {
    use serde_derive::Deserialize;
    use std::path::Path;
    use crate::functions::Result;


    #[derive(Deserialize)]
    struct Telegram {
        api_id: i32,
        api_hash: String,
        bot_token: String,
        owner: i64
    }

    #[derive(Deserialize)]
    struct _Following {
        list: Vec<u64>
    }

    #[derive(Deserialize)]
    struct _Configure {
        telegram: Telegram,
        follow: _Following
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
        pub(crate) api_id: i32,
        pub(crate) api_hash: String,
        pub(crate) bot_token: String,
        pub(crate) owner: i64,
        pub(crate) following: Vec<u64>
    }

    impl Configure {
        pub fn new<P: AsRef<Path>>(path: P) -> Result<Configure> {
            let _configure = _Configure::new(path)?;
            /*let mut following_list: Vec<i128> = Default::default();
            if _configure.follow.list.contains(',') {
                let splits = _configure.follow.list.split(',');
                splits.collect::<Vec<String>>().iter().map(|x| {
                    let id: i128 = x.trim().parse()?;
                    following_list.push(id);
                } );
            } else {
                following_list.push({
                    _configure.follow.list.clone().trim().parse()?
                })
            }*/
            Ok(Configure{
                api_id: _configure.telegram.api_id,
                api_hash: _configure.telegram.api_hash,
                bot_token: _configure.telegram.bot_token,
                owner: _configure.telegram.owner,
                following: _configure.follow.list
            })
        }
    }


}