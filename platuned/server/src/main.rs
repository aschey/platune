mod rpc;
mod server;
mod services;
mod startup;
#[cfg(unix)]
mod unix;

use crate::startup::ServiceHandler;
use daemon_slayer::cli::CliAsync;
use daemon_slayer::cli::ServiceCommand;
use daemon_slayer::logging::tracing_subscriber::util::SubscriberInitExt;
use daemon_slayer::logging::LoggerBuilder;
use daemon_slayer::server::HandlerAsync;
use dotenv::dotenv;
use rpc::*;
use std::error::Error;
use tracing::error;
use tracing::metadata::LevelFilter;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let default_level = if cfg!(feature = "tokio-console") {
        tracing::Level::TRACE
    } else {
        tracing::Level::INFO
    };
    let (logger, guard) = LoggerBuilder::new(ServiceHandler::get_service_name())
        .with_default_log_level(default_level)
        .with_level_filter(LevelFilter::INFO)
        .with_ipc_logger(true)
        .build()?;
    #[cfg(feature = "console")]
    let logger = logger.with(console_subscriber::spawn());

    logger.init();
    match run_async() {
        Ok(()) => Ok(()),
        err @ Err(_) => {
            error!("{err:?}");
            drop(guard);
            err
        }
    }
}

#[tokio::main]
async fn run_async() -> Result<(), Box<dyn Error + Send + Sync>> {
    let cli = CliAsync::for_server(
        ServiceHandler::new(),
        "Platune".to_owned(),
        "platune service".to_owned(),
    );

    let action = cli.action();

    // Don't set panic hook until after logging is set up
    let theme = if action.command == Some(ServiceCommand::Run) {
        color_eyre::config::Theme::new()
    } else {
        color_eyre::config::Theme::dark()
    };
    if action.command == Some(ServiceCommand::Direct) {
        dotenv().ok();
    }
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .add_default_filters()
        .theme(theme)
        .into_hooks();
    eyre_hook.install()?;
    std::panic::set_hook(Box::new(move |pi| {
        error!("{}", panic_hook.panic_report(pi));
    }));

    cli.handle_input().await?;
    Ok(())
}
