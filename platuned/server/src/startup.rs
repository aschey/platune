use crate::server;
use crate::signal_handler::platform::SignalHandler;
#[cfg(windows)]
use crate::windows::service;
use anyhow::Result;
use tokio::sync::broadcast;

async fn run_server() -> Result<()> {
    let (tx, _) = broadcast::channel(32);
    let signal_handler = SignalHandler::start(tx.clone())?;
    server::run_all(tx).await?;
    Ok(signal_handler.close().await?)
}

#[cfg(windows)]
pub async fn start() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "-i" {
        service::install()
    } else if args.len() > 1 && args[1] == "-s" {
        dotenv::from_path(r"C:\Users\asche\code\platune\platuned\server\.env").unwrap();
        service::run()
    } else {
        dotenv::from_path("./.env").unwrap();
        run_server().await
    }
}

#[cfg(unix)]
pub async fn start() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if !(args.len() > 1 && args[1] == "-s") {
        dotenv::from_path("./.env").unwrap();
    }

    run_server().await
}
