use std::env::current_exe;

use daemon_slayer::cli::Cli;
use daemon_slayer::client::cli::ClientCliProvider;
use daemon_slayer::client::config::systemd::SystemdConfig;
use daemon_slayer::client::config::windows::{ServiceAccess, Trustee, WindowsConfig};
use daemon_slayer::client::config::Level;
use daemon_slayer::client::{self};
use daemon_slayer::console::cli::ConsoleCliProvider;
use daemon_slayer::console::{Console, LogSource};
use daemon_slayer::core::BoxedError;
use daemon_slayer::error_handler::cli::ErrorHandlerCliProvider;
use daemon_slayer::error_handler::color_eyre::eyre;
use daemon_slayer::error_handler::ErrorSink;
use daemon_slayer::health_check::cli::HealthCheckCliProvider;
use daemon_slayer::health_check::GrpcHealthCheck;
use daemon_slayer::logging::cli::LoggingCliProvider;
use daemon_slayer::logging::tracing_subscriber::util::SubscriberInitExt;
use daemon_slayer::logging::LoggerBuilder;
use daemon_slayer::process::cli::ProcessCliProvider;
use platuned::{build_info, clap_base_command, main_server_port, service_label};

#[tokio::main]
async fn main() -> Result<(), ErrorSink> {
    let guard = daemon_slayer::logging::init();
    let result = run().await.map_err(|e| ErrorSink::new(eyre::eyre!(e)));
    drop(guard);
    result
}

async fn run() -> Result<(), BoxedError> {
    let label = service_label();
    let mut manager_builder = client::builder(
        label.clone(),
        current_exe()?
            .parent()
            .unwrap()
            .join("platuned")
            .try_into()?,
    )
    .with_description("platune service")
    .with_service_level(Level::User)
    .with_autostart(true)
    .with_windows_config(WindowsConfig::default().with_additional_access(
        Trustee::CurrentUser,
        ServiceAccess::Start | ServiceAccess::Stop | ServiceAccess::ChangeConfig,
    ))
    .with_systemd_config(
        SystemdConfig::default()
            .with_after_target("network.target")
            .with_after_target("network-online.target")
            .with_after_target("NetworkManager.service")
            .with_after_target("systemd-resolved.service"),
    )
    .with_arg(&"run".parse()?);

    if let Ok(()) = dotenvy::from_path("./.env") {
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            manager_builder =
                manager_builder.with_environment_variable("DATABASE_URL", database_url);
        }
        if let Ok(spellfix_lib) = std::env::var("SPELLFIX_LIB") {
            manager_builder =
                manager_builder.with_environment_variable("SPELLFIX_LIB", spellfix_lib);
        }
    }
    let manager = manager_builder.build().await.unwrap();
    let logger_builder = LoggerBuilder::new(label.clone());

    let health_check =
        GrpcHealthCheck::new(format!("http://[::1]:{}", main_server_port()?)).unwrap();

    let console = Console::new(manager.clone(), LogSource::Ipc)
        .await
        .with_health_check(Box::new(health_check.clone()));

    let mut cli = Cli::builder()
        .with_base_command(clap_base_command())
        .with_provider(ClientCliProvider::new(manager.clone()))
        .with_provider(ProcessCliProvider::new(manager.status().await?.pid))
        .with_provider(ConsoleCliProvider::new(console))
        .with_provider(LoggingCliProvider::new(logger_builder))
        .with_provider(ErrorHandlerCliProvider::default())
        .with_provider(HealthCheckCliProvider::new(health_check))
        .with_provider(build_info())
        .initialize()?;

    let (logger, _) = cli.take_provider::<LoggingCliProvider>().get_logger()?;
    logger.init();

    cli.handle_input().await?;
    Ok(())
}
