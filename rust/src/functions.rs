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
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub(crate) mod telegram {

    use grammers_client::{Client, Config, SignInError};
    use grammers_session::FileSession;
    use std::env;
    use std::io::{self, BufRead as _, Write as _w};
    use crate::functions::Result;


    fn console_prompt(message: &str) -> Result<String> {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        stdout.write_all(message.as_bytes())?;
        stdout.flush()?;

        let stdin = io::stdin();
        let mut stdin = stdin.lock();

        let mut line = String::new();
        stdin.read_line(&mut line)?;
        Ok(line)
    }

    pub async fn try_connect_bot(api_id: i32, api_hash: &str, session_name: &str, bot_token: &str) -> Result<Client<FileSession>> {
        _try_connect(api_id, api_hash, session_name, Some(bot_token)).await
    }

    pub async fn try_connect(api_id: i32, api_hash: &str, session_name: &str) -> Result<Client<FileSession>> {
        _try_connect(api_id, api_hash, session_name, None).await
    }

    async fn _try_connect(api_id: i32, api_hash: &str, session_name: &str, bot_token: Option<&str>) -> Result<Client<FileSession>> {
        let mut client = Client::connect(Config {
            session: FileSession::load_or_create(session_name)?,
            api_id,
            api_hash: api_hash.to_string(),
            params: Default::default(),
        })
            .await?;

        if !client.is_authorized().await? {
            println!("Signing in...");
            match bot_token {
                None => {
                    let phone = console_prompt("Enter your phone number: ")?;
                    let token = client.request_login_code(&phone, api_id, &api_hash).await?;
                    let code = console_prompt("Enter the code: ")?;
                    let signed_in = client.sign_in(&token, &code).await;
                    match signed_in {
                        Err(SignInError::PasswordRequired(password_token)) => {
                            let hint = password_token.hint().unwrap();
                            let prompt_message = format!("Enter the password (hint: {}): ", &hint);
                            let password = console_prompt(prompt_message.as_str())?;

                            client
                                .check_password(password_token, password.trim())
                                .await?;
                        }
                        Ok(_) => (),
                        Err(e) => panic!("{}", e),
                    };
                    println!("Signed in!");
                }
                Some(token) => {
                    client.bot_sign_in(token, api_id, api_hash).await?;
                }
            }

            match client.session().save() {
                Ok(_) => {}
                Err(e) => {
                    client.handle().sign_out_disconnect().await?;
                    panic!(
                        "NOTE: failed to save the session, will sign out now: {}",
                        e
                    );
                }
            }
        }

        Ok(client)
    }
}