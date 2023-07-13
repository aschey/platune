use daemon_slayer::{
    build_info::{
        cli::BuildInfoCliProvider,
        vergen_pretty::{vergen_pretty_env, PrettyBuilder, Style},
        Color,
    },
    cli::Cli,
    client::{
        self,
        cli::ClientCliProvider,
        config::{
            systemd::SystemdConfig,
            windows::{ServiceAccess, Trustee, WindowsConfig},
            Level,
        },
    },
    console::{cli::ConsoleCliProvider, Console, LogSource},
    core::{BoxedError, Label},
    error_handler::{cli::ErrorHandlerCliProvider, color_eyre::eyre, ErrorSink},
    health_check::{cli::HealthCheckCliProvider, GrpcHealthCheck},
    logging::{
        cli::LoggingCliProvider, tracing_subscriber::util::SubscriberInitExt, LoggerBuilder,
    },
    process::cli::ProcessCliProvider,
};
use std::env::current_exe;

#[tokio::main]
async fn main() -> Result<(), ErrorSink> {
    let guard = daemon_slayer::logging::init();
    let result = run().await.map_err(|e| ErrorSink::new(eyre::eyre!(e)));
    drop(guard);
    result
}

async fn run() -> Result<(), BoxedError> {
    let label: Label = "com.platune.platuned".parse()?;
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

    let health_check = GrpcHealthCheck::new("http://[::1]:50051").unwrap();

    let console = Console::new(manager.clone(), LogSource::Ipc)
        .await
        .with_health_check(Box::new(health_check.clone()));

    let pretty = PrettyBuilder::default()
        .env(vergen_pretty_env!())
        .key_style(Style::default().fg(Color::Cyan).bold())
        .value_style(Style::default())
        .category(false)
        .build()
        .unwrap();

    let mut cli = Cli::builder()
        .with_provider(ClientCliProvider::new(manager.clone()))
        .with_provider(ProcessCliProvider::new(manager.info().await?.pid))
        .with_provider(ConsoleCliProvider::new(console))
        .with_provider(LoggingCliProvider::new(logger_builder))
        .with_provider(ErrorHandlerCliProvider::default())
        .with_provider(HealthCheckCliProvider::new(health_check))
        .with_provider(BuildInfoCliProvider::new(pretty))
        .initialize()?;

    let (logger, _) = cli.take_provider::<LoggingCliProvider>().get_logger()?;
    logger.init();

    cli.handle_input().await?;
    Ok(())
}
