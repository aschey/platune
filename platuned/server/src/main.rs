#![feature(ip)]
mod cert_gen;
mod ipc_stream;
mod rpc;
mod server;
mod services;
mod startup;

use daemon_slayer::cli::Cli;
use daemon_slayer::core::BoxedError;
use daemon_slayer::error_handler::ErrorSink;
use daemon_slayer::error_handler::cli::ErrorHandlerCliProvider;
use daemon_slayer::error_handler::color_eyre::eyre;
use daemon_slayer::logging::cli::LoggingCliProvider;
#[cfg(feature = "tokio-console")]
use daemon_slayer::logging::tracing_subscriber::prelude::*;
use daemon_slayer::logging::tracing_subscriber::util::SubscriberInitExt;
use daemon_slayer::logging::{EnvConfig, LoggerBuilder};
use daemon_slayer::notify::notification::Notification;
use daemon_slayer::server::Handler;
use daemon_slayer::server::cli::ServerCliProvider;
use dotenvy::dotenv;
use platuned::{build_info, clap_base_command};
use rpc::*;

use crate::startup::ServiceHandler;

#[tokio::main]
async fn main() -> Result<(), ErrorSink> {
    let guard = daemon_slayer::logging::init();
    let result = run().await.map_err(|e| ErrorSink::new(eyre::eyre!(e)));
    drop(guard);
    result
}

async fn run() -> Result<(), BoxedError> {
    let default_level = if cfg!(feature = "tokio-console") {
        // TODO: get rid of log spam in our normal log targets when enabling this
        // we should only send the spammy logs to the tokio console
        tracing::Level::TRACE
    } else {
        tracing::Level::INFO
    };

    let logger_builder = LoggerBuilder::new(ServiceHandler::label())
        .with_env_config(
            EnvConfig::new("PLATUNE_LOG".to_string()).with_default(default_level.into()),
        )
        // Lofty spams warning logs for metadata parsing issues
        // TODO: make a setting to control this
        .with_env_filter_directive("lofty=error".parse()?);

    let mut cli = Cli::builder()
        .with_base_command(clap_base_command())
        .with_provider(ServerCliProvider::<ServiceHandler>::new(
            &"run".parse().unwrap(),
        ))
        .with_provider(LoggingCliProvider::new(logger_builder))
        .with_provider(
            ErrorHandlerCliProvider::default().with_notification(
                Notification::new(ServiceHandler::label())
                    .summary("The platune service encountered a fatal error"),
            ),
        )
        .with_provider(build_info())
        .initialize()?;

    // TODO: use get_logger_with_reload when we have config file support
    let logger = cli.take_provider::<LoggingCliProvider>().get_logger()?;

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
