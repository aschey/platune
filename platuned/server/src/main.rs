mod rpc;
mod server;
mod services;
mod startup;
#[cfg(unix)]
mod unix;

use daemon_slayer::cli::Action;
use daemon_slayer::cli::CliAsync;
use daemon_slayer::cli::CliHandlerAsync;
use daemon_slayer::client::Level;
use daemon_slayer::client::Manager;
use daemon_slayer::client::ServiceManager;
use daemon_slayer::logging::LoggerBuilder;
use daemon_slayer::logging::LoggerGuard;
use daemon_slayer_server::HandlerAsync;
use directories::ProjectDirs;
use rpc::*;
use std::io::stdout;
use std::process::exit;
use time::format_description::well_known;
use time::UtcOffset;
use tracing::error;
use tracing::info;
use tracing::log::warn;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{
    filter::LevelFilter, fmt::Layer, layer::SubscriberExt, EnvFilter, Layer as SubscriberLayer,
};

use crate::startup::ServiceHandler;

fn main() {
    let logger_builder = LoggerBuilder::new(ServiceHandler::get_service_name());

    run_async(logger_builder);
}

#[tokio::main]
async fn run_async(logger_builder: LoggerBuilder) {
    let manager = ServiceManager::builder(ServiceHandler::get_service_name())
        .with_description("platune service")
        .with_service_level(Level::User)
        .with_args(["run"])
        .build()
        .unwrap();
    let cli = CliAsync::<ServiceHandler>::new(manager);
    let mut logger_guard: Option<LoggerGuard> = None;

    if cli.action_type() == Action::Server {
        let (logger, guard) = logger_builder.build();
        logger_guard = Some(guard);
        logger.init();
        dotenv::from_path("./.env").unwrap();
        let path = std::env::var("DATABASE_URL").unwrap();
        let db = libplatune_management::database::Database::connect(path, true)
            .await
            .unwrap();
        db.migrate().await.unwrap();
    }
    // Don't set panic hook until after logging is set up
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .add_default_filters()
        .into_hooks();
    eyre_hook.install().unwrap();
    std::panic::set_hook(Box::new(move |pi| {
        error!("{}", panic_hook.panic_report(pi));
    }));
    if let Err(e) = cli.handle_input().await {
        error!("{:?}", e);
        // Drop guards to ensure logs are flushed
        drop(logger_guard);
        exit(1);
    }
}
