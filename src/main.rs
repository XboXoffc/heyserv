use std::sync::Arc;
use anyhow::{Result, bail};
use russh::*;
use russh::client;
use russh::keys::PublicKeyOrCertificate;


#[tokio::main]
async fn main() -> Result<()> {
    let host = "localhost";
    let port:u16 = 22;
    let username = "root";
    let password = "12345678";



    let config = client::Config::default();
    let config = Arc::new(config);

    let mut session = client::connect(config, (host, port), Client).await?;
    let auth = session.authenticate_password(username, password).await?;

    if !auth.success() {
        bail!("auth is not success...");
    }
    println!("auth success!");

    let mut channel = session.channel_open_session().await?;
    channel.exec(true, "uptime").await?;

    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::Data { ref data } => {
                println!("{}", String::from_utf8_lossy(data));
            }
            ChannelMsg::ExitStatus { exit_status } => {
                println!("ExitStatus: {}", exit_status);
            }
            _ => {}
        }
    }



    Ok(())

}

struct Client;

impl client::Handler for Client{ 
    type Error = russh::Error;
    
    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}
