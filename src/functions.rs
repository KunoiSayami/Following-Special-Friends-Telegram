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
pub(crate) mod telegram {

    use anyhow::Result;
    use grammers_client::{Client, Config, InitParams, SignInError};
    use grammers_session::Session;
    use serde_derive::Serialize;
    use std::env;
    use std::io::{self, BufRead as _, Write as _w};

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

    /*pub async fn try_connect_bot(api_id: i32, api_hash: &str, session_name: &str, bot_token: &str) -> Result<Client<FileSession>> {
        _try_connect(api_id, api_hash, session_name, Some(bot_token)).await
    }*/

    pub async fn try_connect(api_id: i32, api_hash: &str, session_path: &str) -> Result<Client> {
        _try_connect(api_id, api_hash, session_path, None).await
    }

    fn get_init_params<T>(device_model: T) -> InitParams
    where
        T: Into<String>,
    {
        InitParams {
            device_model: device_model.into(),
            // https://stackoverflow.com/a/62409338
            system_version: format!(
                "{} {} {}",
                env::consts::FAMILY,
                env::consts::OS,
                env::consts::ARCH
            ),
            ..Default::default()
        }
    }

    async fn _try_connect(
        api_id: i32,
        api_hash: &str,
        session_path: &str,
        bot_token: Option<&str>,
    ) -> Result<Client> {
        //let session_file_path = Path::new("data").join(format!("{}.session", session_name));
        let client = Client::connect(Config {
            session: Session::load_file_or_create(session_path.clone())?,
            api_id,
            api_hash: api_hash.to_string(),
            // https://stackoverflow.com/a/27841363
            params: get_init_params(option_env!("CARGO_PKG_NAME").unwrap_or("unknown")),
        })
        .await?;

        if !client.is_authorized().await? {
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

            match client.session().save_to_file(session_path) {
                Ok(_) => {}
                Err(e) => {
                    client.sign_out_disconnect().await?;
                    panic!("NOTE: failed to save the session, will sign out now: {}", e);
                }
            }
        }

        Ok(client)
    }

    #[derive(Clone, Debug, Serialize)]
    pub struct SendMessageParameters {
        chat_id: i64,
        text: String,
        parse_mode: String,
    }

    impl SendMessageParameters {
        pub fn new<T>(chat_id: i64, text: T) -> SendMessageParameters
        where
            T: Into<String>,
        {
            SendMessageParameters {
                chat_id,
                text: text.into(),
                parse_mode: String::from("markdown"),
            }
        }
    }
}
