# Following-Special-Friends-Telegram

A bot that can monitor specify user send message to your common group

## Configure

Copy `config.toml.default` to `config.toml`, this configure file should store in `data` folder
```toml
[telegram]
# If you don't have api_id and api_hash, obtain them from https://my.telegram.org/apps
api_id = 0
api_hash = ""
# You can create bot from [@Botfather](https://t.me/botfather)
# Please note: You should contact your bot with your owner account use `/start` command at least once
bot_token = ""
# Set as your own telegram user ID
owner = 0
# Optional: User can specify their own server build from https://github.com/tdlib/telegram-bot-api
# api_address = "https://api.telegram.org"

[follow]
# Insert your following user id to this list
list = []
```

## Build

Just use cargo to build.

```shell
cargo build --relase
```

## License

[![](https://www.gnu.org/graphics/agplv3-155x51.png)](https://www.gnu.org/licenses/agpl-3.0.txt)

Copyright (C) 2020-2021 KunoiSayami

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
