mod rpc;
mod server;
mod services;
mod startup;
#[cfg(unix)]
mod unix;

use daemon_slayer::cli::ActionType;
use daemon_slayer::cli::CliAsync;
use daemon_slayer::cli::ServiceCommand;
use daemon_slayer::client::health_check::GrpcHealthCheckAsync;
use daemon_slayer::client::Level;
use daemon_slayer::client::Manager;
use daemon_slayer::client::ServiceManager;
use daemon_slayer::logging::tracing_subscriber::util::SubscriberInitExt;
use daemon_slayer::logging::LoggerBuilder;
use daemon_slayer::logging::LoggerGuard;
use daemon_slayer::server::HandlerAsync;
use rpc::*;
use std::process::exit;
use tracing::error;
use tracing::metadata::LevelFilter;

use crate::startup::ServiceHandler;

fn main() {
    let logger_builder = LoggerBuilder::new(ServiceHandler::get_service_name());

    run_async(logger_builder);
}

#[tokio::main]
async fn run_async(logger_builder: LoggerBuilder) {
    let mut manager_builder = ServiceManager::builder(ServiceHandler::get_service_name())
        .with_description("platune service")
        .with_service_level(Level::User)
        .with_args(["run"]);
    if let Ok(()) = dotenv::from_path("./.env") {
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            manager_builder = manager_builder.with_env_var("DATABASE_URL", database_url);
        }
        if let Ok(spellfix_lib) = std::env::var("SPELLFIX_LIB") {
            manager_builder = manager_builder.with_env_var("SPELLFIX_LIB", spellfix_lib);
        }
    }
    let manager = manager_builder.build().unwrap();
    let cli = CliAsync::builder_for_all(manager, ServiceHandler::new())
        .with_health_check(Box::new(
            GrpcHealthCheckAsync::new("http://[::1]:50051").unwrap(),
        ))
        .build();
    let mut logger_guard: Option<LoggerGuard> = None;

    let action = cli.action();
    if action.action_type == ActionType::Server {
        let default_level = if cfg!(feature = "tokio-console") {
            tracing::Level::TRACE
        } else {
            tracing::Level::INFO
        };
        let (logger, guard) = logger_builder
            .with_default_log_level(default_level)
            .with_level_filter(LevelFilter::INFO)
            .with_ipc_logger(true)
            .build()
            .unwrap();
        #[cfg(feature = "console")]
        let logger = logger.with(console_subscriber::spawn());
        logger_guard = Some(guard);
        logger.init();

        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            let db = libplatune_management::database::Database::connect(database_url, true)
                .await
                .unwrap();
            db.migrate().await.unwrap();
        }
    }
    // Don't set panic hook until after logging is set up
    let theme = if action.command == Some(ServiceCommand::Run) {
        color_eyre::config::Theme::new()
    } else {
        color_eyre::config::Theme::dark()
    };
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .add_default_filters()
        .theme(theme)
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
