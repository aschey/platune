mod ipc_stream;
mod rpc;
mod server;
mod services;
mod startup;

use crate::startup::ServiceHandler;
#[cfg(feature = "tokio-console")]
use daemon_slayer::logging::tracing_subscriber::prelude::*;
use daemon_slayer::{
    build_info::{
        cli::BuildInfoCliProvider,
        vergen_pretty::{vergen_pretty_env, PrettyBuilder, Style},
        Color,
    },
    cli::Cli,
    core::BoxedError,
    error_handler::{cli::ErrorHandlerCliProvider, color_eyre::eyre, ErrorSink},
    logging::{
        self, cli::LoggingCliProvider, tracing_subscriber::util::SubscriberInitExt, LogLevel,
        LoggerBuilder,
    },
    notify::notification::Notification,
    server::{cli::ServerCliProvider, Handler},
};
use dotenvy::dotenv;
use rpc::*;

#[tokio::main]
async fn main() -> Result<(), ErrorSink> {
    let guard = daemon_slayer::logging::init();
    let result = run().await.map_err(|e| ErrorSink::new(eyre::eyre!(e)));
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

    let pretty = PrettyBuilder::default()
        .env(vergen_pretty_env!())
        .key_style(Style::default().fg(Color::Cyan).bold())
        .value_style(Style::default())
        .category(false)
        .build()
        .unwrap();

    let mut cli = Cli::builder()
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
        .with_provider(BuildInfoCliProvider::new(pretty))
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
