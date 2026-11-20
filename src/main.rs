use std::sync::Arc;
use anyhow::{Context, Ok, Result, bail};
use russh::ChannelMsg;
use russh::client;
use russh::keys::PublicKeyOrCertificate;
use serde::{Deserialize};
use toml;
use std::fs;
use clap::Parser;

const CONFIG_PATH:&str = "config.toml";

struct Client;

impl client::Handler for Client{ 
    type Error = anyhow::Error;
    
    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}


#[derive(Debug, Deserialize)]
struct ServerConfig{
    host:String,
    port:u16,
    username:String,
    password:String,
}

#[derive(Debug, Deserialize)]
struct ServicesConfig{
    list:Vec<String>    
}

#[derive(Debug, Deserialize)]
struct PingConfig{
    hosts:Vec<String>
}

#[derive(Debug, Deserialize)]
struct Config{
    server:ServerConfig,
    services:ServicesConfig,
    ping:PingConfig,
}

#[derive(Parser, Debug)]
#[command(version, long_about = None)]
struct Args{
    #[arg(short('H'), long)]
    host: Option<String>,

    #[arg(short, long)]
    port: Option<u16>,

    #[arg(short, long)]
    username: Option<String>,

    #[arg(short('P'), long)]
    password: Option<String>,
}


#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config(CONFIG_PATH).await?;

    let args: Args = Args::parse();
    let host = args.host.unwrap_or(config.server.host);
    let port = args.port.unwrap_or(config.server.port);
    let username = args.username.unwrap_or(config.server.username);
    let password = args.password.unwrap_or(config.server.password);

    Ok(ssh_exec(&host, &port, &username, &password).await?)
}

async fn ssh_exec(host:&str, port: &u16, username:&str, password:&str) -> Result<()> {
    let config = client::Config::default();
    let config = Arc::new(config);

    let mut session = client::connect(config, (host, *port), Client).await?;
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

async fn load_config(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Не удалось прочитать файл конфига: {}", path))?;
    
    let config: Config = toml::from_str(&content)
        .context("Ошибка парсинга TOML")?;
    
    Ok(config)
}