use std::env::current_exe;

use auto_launch::AutoLaunchBuilder;
use clap::{FromArgMatches, Subcommand};
use daemon_slayer::cli::{Cli, InputState};
use daemon_slayer::client::cli::ClientCliProvider;
use daemon_slayer::client::config::Level;
use daemon_slayer::client::config::systemd::SystemdConfig;
use daemon_slayer::client::config::windows::{ServiceAccess, Trustee, WindowsConfig};
use daemon_slayer::client::{self};
use daemon_slayer::console::cli::ConsoleCliProvider;
use daemon_slayer::console::{Console, LogSource};
use daemon_slayer::core::BoxedError;
use daemon_slayer::error_handler::ErrorSink;
use daemon_slayer::error_handler::cli::ErrorHandlerCliProvider;
use daemon_slayer::error_handler::color_eyre::eyre;
use daemon_slayer::health_check::GrpcHealthCheck;
use daemon_slayer::health_check::cli::HealthCheckCliProvider;
use daemon_slayer::logging::LoggerBuilder;
use daemon_slayer::logging::cli::LoggingCliProvider;
use daemon_slayer::logging::tracing_subscriber::util::SubscriberInitExt;
use daemon_slayer::process::cli::ProcessCliProvider;
use platuned::{build_info, clap_base_command, main_server_port, service_label};
use which::which;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorSink> {
    let guard = daemon_slayer::logging::init();
    let result = run().await.map_err(|e| ErrorSink::new(eyre::eyre!(e)));
    drop(guard);
    result
}

async fn run() -> Result<(), BoxedError> {
    let label = service_label();
    let exe = current_exe()?;
    let exe_parent = exe.parent().unwrap();
    let mut manager_builder =
        client::builder(label.clone(), exe_parent.join("platuned").try_into()?)
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
        manager_builder = manager_builder
            .with_environment_variable_if_exists("DATABASE_URL")
            .with_environment_variable_if_exists("SPELLFIX_LIB")
            .with_environment_variable_if_exists("PLATUNE_ENABLE_TLS")
            .with_environment_variable_if_exists("PLATUNE_ENABLE_CLIENT_TLS")
            .with_environment_variable_if_exists("PLATUNE_HOSTS")
            .with_environment_variable_if_exists("PLATUNE_GLOBAL_FILE_URL")
            .with_environment_variable_if_exists("PLATUNE_IP_HEADER")
            .with_environment_variable_if_exists("PLATUNE_MTLS_CLIENT_CERT_PATH")
            .with_environment_variable_if_exists("PLATUNE_MTLS_CLIENT_KEY_PATH");
    }
    if let Ok(yt_dlp_path) = which("yt-dlp") {
        manager_builder =
            manager_builder.with_environment_variable("YT_DLP_PATH", yt_dlp_path.to_string_lossy());
    }
    if let Ok(ffmpeg_path) = which("ffmpeg") {
        manager_builder =
            manager_builder.with_environment_variable("FFMPEG_PATH", ffmpeg_path.to_string_lossy());
    }
    let manager = manager_builder.build().await.unwrap();
    let logger_builder = LoggerBuilder::new(label.clone());

    let health_check =
        GrpcHealthCheck::new(format!("http://[::1]:{}", main_server_port()?)).unwrap();

    let console = Console::new(manager.clone(), LogSource::Ipc)
        .await
        .with_health_check(Box::new(health_check.clone()));

    let base_command = clap_base_command();
    let mut cli = Cli::builder()
        .with_base_command(TrayCommand::augment_subcommands(base_command))
        .with_provider(ClientCliProvider::new(manager.clone()))
        .with_provider(ProcessCliProvider::new(manager.pid().await?))
        .with_provider(ConsoleCliProvider::new(console))
        .with_provider(LoggingCliProvider::new(logger_builder))
        .with_provider(ErrorHandlerCliProvider::default())
        .with_provider(HealthCheckCliProvider::new(health_check))
        .with_provider(build_info())
        .initialize()?;

    let logger = cli.take_provider::<LoggingCliProvider>().get_logger()?;
    logger.init();

    let (state, matches) = cli.handle_input().await?;
    if state == InputState::Unhandled {
        if let Ok(TrayCommand::Tray(Tray { tray })) = TrayCommand::from_arg_matches(&matches) {
            let auto_launch = AutoLaunchBuilder::new()
                .set_app_name("platune-tray")
                .set_app_path(&exe_parent.join("platune-tray").to_string_lossy())
                .set_use_launch_agent(true)
                .build()
                .unwrap();
            match tray {
                TrayValue::Enable => {
                    auto_launch.enable()?;
                }
                TrayValue::Disable => {
                    auto_launch.disable()?;
                }
            }
        }
    }
    Ok(())
}

#[derive(clap::Subcommand)]
enum TrayCommand {
    Tray(Tray),
}

#[derive(clap::Args, Clone)]
struct Tray {
    #[command(subcommand)]
    tray: TrayValue,
}

#[derive(clap::Subcommand, Clone)]
enum TrayValue {
    Enable,
    Disable,
}
