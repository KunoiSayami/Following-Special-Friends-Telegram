
pub(crate) mod telegram {

    use grammers_client::{Client, Config, SignInError};
    use grammers_session::FileSession;
    use log;
    use std::env;
    use std::io::{self, BufRead as _, Write as _w};

    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    fn prompt(message: &str) -> Result<String> {
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

    pub async fn try_connect(api_id: i32, api_hash: &str, session_name: &str) -> Result<Client<FileSession>> {
        let mut client = Client::connect(Config {
            session: FileSession::load_or_create(session_name)?,
            api_id,
            api_hash: api_hash.to_string(),
            params: Default::default(),
        })
            .await?;

        if !client.is_authorized().await? {
            println!("Signing in...");
            let phone = prompt("Enter your phone number (international format): ")?;
            let token = client.request_login_code(&phone, api_id, &api_hash).await?;
            let code = prompt("Enter the code you received: ")?;
            let signed_in = client.sign_in(&token, &code).await;
            match signed_in {
                Err(SignInError::PasswordRequired(password_token)) => {
                    // Note: this `prompt` method will echo the password in the console.
                    //       Real code might want to use a better way to handle this.
                    let hint = password_token.hint().unwrap();
                    let prompt_message = format!("Enter the password (hint {}): ", &hint);
                    let password = prompt(prompt_message.as_str())?;

                    client
                        .check_password(password_token, password.trim())
                        .await?;
                }
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            };
            println!("Signed in!");
            match client.session().save() {
                Ok(_) => {}
                Err(e) => {
                    client.handle().sign_out_disconnect().await?;
                    panic!(
                        "NOTE: failed to save the session, will sign out and now: {}",
                        e
                    );
                }
            }
        }

        Ok(client)
    }
}