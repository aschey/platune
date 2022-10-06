mod rpc;
mod server;
mod services;
mod startup;
#[cfg(unix)]
mod unix;

use crate::startup::ServiceHandler;
use daemon_slayer::cli::CliAsync;
use daemon_slayer::cli::ServiceCommand;
#[cfg(feature = "console")]
use daemon_slayer::logging::tracing_subscriber::prelude::*;
use daemon_slayer::logging::tracing_subscriber::util::SubscriberInitExt;
use daemon_slayer::server::HandlerAsync;
use dotenvy::dotenv;
use rpc::*;
use std::error::Error;
use tracing::metadata::LevelFilter;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    daemon_slayer::logging::init_local_time();
    run_async()
}

#[tokio::main]
async fn run_async() -> Result<(), Box<dyn Error + Send + Sync>> {
    let default_level = if cfg!(feature = "tokio-console") {
        tracing::Level::TRACE
    } else {
        tracing::Level::INFO
    };
    let cli = CliAsync::for_server(
        ServiceHandler::new(),
        "platune".to_owned(),
        "Platune".to_owned(),
        "platune service".to_owned(),
    );
    let (logger, _guard) = cli
        .configure_logger()
        .with_default_log_level(default_level)
        .with_level_filter(LevelFilter::INFO)
        .with_ipc_logger(true)
        .build()?;

    #[cfg(feature = "console")]
    let logger = logger.with(console_subscriber::spawn());

    logger.init();
    // Don't set panic hook until after logging is set up
    cli.configure_error_handler().install()?;

    let action = cli.action();

    if action.command == Some(ServiceCommand::Direct) {
        dotenv().ok();
    }

    cli.handle_input().await?;
    Ok(())
}
