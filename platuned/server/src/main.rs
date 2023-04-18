mod ipc_stream;
mod rpc;
mod server;
mod services;
mod startup;

use crate::startup::ServiceHandler;
#[cfg(feature = "console")]
use daemon_slayer::logging::tracing_subscriber::prelude::*;
use daemon_slayer::{
    cli::Cli,
    core::BoxedError,
    error_handler::{cli::ErrorHandlerCliProvider, ErrorSink},
    logging::{
        self, cli::LoggingCliProvider, tracing_subscriber::util::SubscriberInitExt, LogLevel,
        LoggerBuilder,
    },
    server::{cli::ServerCliProvider, Handler},
};
use dotenvy::dotenv;
use rpc::*;

#[tokio::main]
async fn main() -> Result<(), ErrorSink> {
    let guard = daemon_slayer::logging::init();
    let result = run().await.map_err(ErrorSink::from_error);
    drop(guard);
    result
}

async fn run() -> Result<(), BoxedError> {
    let default_level = if cfg!(feature = "tokio-console") {
        tracing::Level::TRACE
    } else {
        tracing::Level::INFO
    };

    let logger_builder =
        LoggerBuilder::new(ServiceHandler::label()).with_config(logging::UserConfig {
            log_level: LogLevel(default_level),
        });

    let mut cli = Cli::builder()
        .with_provider(ServerCliProvider::<ServiceHandler>::new(
            &"run".parse().unwrap(),
        ))
        .with_provider(LoggingCliProvider::new(logger_builder))
        .with_provider(ErrorHandlerCliProvider::default())
        .initialize()?;

    let (logger, _) = cli.take_provider::<LoggingCliProvider>().get_logger()?;
    #[cfg(feature = "tokio-console")]
    let logger = logger.with(console_subscriber::spawn());
    logger.init();

    let matches = cli.get_matches();
    if matches.subcommand().is_none() {
        dotenv().ok();
    }

    cli.handle_input().await?;
    Ok(())
}
