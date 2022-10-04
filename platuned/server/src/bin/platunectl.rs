use std::error::Error;

use daemon_slayer::{
    cli::CliAsync,
    client::{
        config::SystemdConfig, health_check::GrpcHealthCheckAsync, Level, Manager, ServiceManager,
    },
    logging::tracing_subscriber::util::SubscriberInitExt,
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
            SystemdConfig::new()
                .with_after_target("network.target")
                .with_after_target("network-online.target")
                .with_after_target("NetworkManager.service")
                .with_after_target("systemd-resolved.service"),
        )
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
    let cli = CliAsync::builder_for_client(manager)
        .with_health_check(Box::new(
            GrpcHealthCheckAsync::new("http://[::1]:50051").unwrap(),
        ))
        .build();
    let (logger, _guard) = cli.configure_logger().build()?;
    logger.init();

    cli.configure_error_handler().install()?;

    cli.handle_input().await?;
    Ok(())
}
