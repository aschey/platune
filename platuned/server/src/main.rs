mod rpc;
mod server;
mod services;
mod startup;
mod sync_handler;
mod sync_handler_builder;
mod sync_handler_client;
mod sync_processor;
#[cfg(unix)]
mod unix;

use crate::startup::ServiceHandler;
#[cfg(feature = "tokio-console")]
use daemon_slayer::logging::tracing_subscriber::prelude::*;
use daemon_slayer::{
    cli::Cli,
    error_handler::{cli::ErrorHandlerCliProvider, ErrorHandler},
    logging::{
        cli::LoggingCliProvider, tracing_subscriber::util::SubscriberInitExt, LoggerBuilder,
    },
    server::cli::ServerCliProvider,
};
use dotenvy::dotenv;
use rpc::*;
use std::{error::Error, time::Duration};
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

    let logger_builder = LoggerBuilder::new("platuned")
        .with_ipc_logger(true)
        .with_default_log_level(default_level)
        .with_level_filter(LevelFilter::INFO);
    let logging_provider = LoggingCliProvider::new(logger_builder);

    let cli = Cli::builder()
        .with_default_server_commands()
        .with_provider(ServerCliProvider::<ServiceHandler>::default())
        .with_provider(logging_provider.clone())
        .with_provider(ErrorHandlerCliProvider::default())
        .initialize();

    let (logger, _guard) = logging_provider.get_logger();
    #[cfg(feature = "tokio-console")]
    let logger = logger.with(console_subscriber::spawn());

    logger.init();

    let matches = cli.get_matches();
    if matches.subcommand().is_none() {
        dotenv().ok();
    }

    cli.handle_input().await;
    Ok(())
}
