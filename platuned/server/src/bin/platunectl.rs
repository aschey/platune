use std::error::Error;

use daemon_slayer::{
    cli::Cli,
    client::{cli::ClientCliProvider, config::SystemdConfig, Level, Manager, ServiceManager},
    console::{cli::ConsoleCliProvider, Console},
    error_handler::{cli::ErrorHandlerCliProvider, ErrorHandler},
    health_check::{cli::HealthCheckCliProvider, GrpcHealthCheck},
    logging::{
        cli::LoggingCliProvider, tracing_subscriber::util::SubscriberInitExt, LoggerBuilder,
    },
};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    daemon_slayer::logging::init_local_time();
    run_async()
}

#[tokio::main]
async fn run_async() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut manager_builder = ServiceManager::builder("platuned")
        .with_description("platune service")
        .with_service_level(Level::User)
        .with_autostart(true)
        .with_systemd_config(
            SystemdConfig::default()
                .with_after_target("network.target")
                .with_after_target("network-online.target")
                .with_after_target("NetworkManager.service")
                .with_after_target("systemd-resolved.service"),
        )
        .with_args(["run"]);

    if let Ok(()) = dotenvy::from_path("./.env") {
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            manager_builder = manager_builder.with_env_var("DATABASE_URL", database_url);
        }
        if let Ok(spellfix_lib) = std::env::var("SPELLFIX_LIB") {
            manager_builder = manager_builder.with_env_var("SPELLFIX_LIB", spellfix_lib);
        }
    }
    let manager = manager_builder.build().unwrap();

    let health_check = GrpcHealthCheck::new("http://[::1]:50051").unwrap();

    let mut console = Console::new(manager.clone());
    console.add_health_check(Box::new(health_check.clone()));

    let logger_builder = LoggerBuilder::new("platuned");
    let logging_provider = LoggingCliProvider::new(logger_builder);

    let cli = Cli::builder()
        .with_default_client_commands()
        .with_provider(ClientCliProvider::new(manager.clone()))
        .with_provider(ConsoleCliProvider::new(console))
        .with_provider(HealthCheckCliProvider::new(health_check))
        .with_provider(logging_provider.clone())
        .with_provider(ErrorHandlerCliProvider::default())
        .initialize();

    let (logger, _guard) = logging_provider.get_logger();
    logger.init();

    cli.handle_input().await;
    Ok(())
}
