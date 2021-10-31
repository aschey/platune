mod management;
mod player;
mod signal_handler;
#[cfg(unix)]
mod unix;
use crate::management_server::ManagementServer;
use crate::player_server::PlayerServer;
use crate::unix::unix_stream::UnixStream;
use anyhow::Context;
use anyhow::Result;
use libplatune_management::config::Config;
use libplatune_management::database::Database;
use libplatune_management::manager::Manager;
use libplatune_player::platune_player::PlatunePlayer;
use management::ManagementImpl;
use player::PlayerImpl;
use rpc::*;
use std::net::SocketAddr;
use std::panic;
use std::sync::Arc;
use tokio::sync::broadcast;
use tonic::transport::Server;
use tracing::error;
use tracing::info;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

pub mod rpc {
    tonic::include_proto!("player_rpc");
    tonic::include_proto!("management_rpc");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("rpc_descriptor");
}

#[cfg(windows)]
mod service {
    use anyhow::{Context, Result};
    use std::{
        ffi::{OsStr, OsString},
        time::Duration,
    };
    use tokio::runtime::Runtime;
    use tracing::error;
    use windows_service::{
        service::{
            ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl,
            ServiceExitCode, ServiceInfo, ServiceStartType, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
        service_manager::{ServiceManager, ServiceManagerAccess},
    };

    use crate::run_server;
    windows_service::define_windows_service!(service_main, handle_service_main);

    const SERVICE_NAME: &str = "platuned";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    pub fn run() -> Result<()> {
        service_dispatcher::start(SERVICE_NAME, service_main)
            .with_context(|| "Error starting service")
    }

    pub fn install() -> Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
            .with_context(|| "Error connecting to service database")?;
        let service_access =
            ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
        if let Ok(service) = service_manager.open_service(SERVICE_NAME, service_access) {
            let status = service
                .query_status()
                .with_context(|| "Error querying service status")?;
            if status.current_state == ServiceState::Running {
                service.stop().with_context(|| "Error stopping service")?;
            }
            service.delete().with_context(|| "Error deleting service")?;
        }

        let service_binary_path =
            ::std::env::current_exe().with_context(|| "Error getting current exe path")?;

        let service_info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from(SERVICE_NAME),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::OnDemand,
            error_control: ServiceErrorControl::Normal,
            executable_path: service_binary_path,
            launch_arguments: vec![OsString::from("-s")],
            dependencies: vec![],
            account_name: None, // run as System
            account_password: None,
        };
        let service = service_manager.create_service(
            &service_info,
            ServiceAccess::CHANGE_CONFIG | ServiceAccess::START,
        )?;
        service
            .set_description("platune service")
            .with_context(|| "Unable to set service description")?;
        service
            .start(&[OsStr::new("Started")])
            .with_context(|| "Unable to start service")?;

        Ok(())
    }

    pub fn handle_service_main(_arguments: Vec<OsString>) {
        let (event_tx, event_rx) = tokio::sync::mpsc::channel(32);

        // Define system service event handler that will be receiving service events.
        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                // Notifies a service to report its current status information to the service
                // control manager. Always return NoError even if not implemented.
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

                // Handle stop
                ServiceControl::Stop => {
                    if let Err(e) = event_tx.try_send(()) {
                        error!("Error sending stop signal {:?}", e);
                    }
                    ServiceControlHandlerResult::NoError
                }

                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        // Register system service event handler.
        // The returned status handle should be used to report service status changes to the system.
        let status_handle = match service_control_handler::register(SERVICE_NAME, event_handler) {
            Ok(handle) => handle,
            Err(e) => {
                error!("Error registering service control handler {:?}", e);
                return;
            }
        };

        let rt = match Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                error!("Error starting tokio runtime {:?}", e);
                return;
            }
        };

        // Tell the system that service is running
        if let Err(e) = status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        }) {
            error!("Error changing service status to 'running' {:?}", e);
            return;
        }

        let exit_code = match rt.block_on(async { run_server(Some(event_rx)).await }) {
            Ok(()) => 0,
            Err(e) => {
                error!("Error running server {:?}", e);
                1
            }
        };

        // Tell the system that service has stopped.
        if let Err(e) = status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(exit_code),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        }) {
            error!("Unable to stop service {:?}", e);
        }
    }
}

fn init_logging() {
    let proj_dirs = directories::ProjectDirs::from("", "", "platune")
        .expect("Unable to find a valid home directory");
    let file_appender = tracing_appender::rolling::hourly(proj_dirs.cache_dir(), "platuned.log");
    let (non_blocking_stdout, _stdout_guard) = tracing_appender::non_blocking(std::io::stdout());
    let (non_blocking_file, _file_guard) = tracing_appender::non_blocking(file_appender);

    let collector = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with({
            #[allow(clippy::let_and_return)]
            let layer = Layer::new()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_ansi(false)
                .with_writer(non_blocking_file);

            #[cfg(windows)]
            layer.with_timer(LocalTime::rfc_3339());

            layer
        })
        .with({
            #[allow(clippy::let_and_return)]
            let layer = Layer::new()
                .pretty()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_writer(non_blocking_stdout);

            #[cfg(windows)]
            layer.with_timer(LocalTime::rfc_3339());

            layer
        });

    tracing::subscriber::set_global_default(collector)
        .expect("Unable to set global tracing subscriber");
}

fn set_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            error!("panic occurred: {:?}", s);
        } else {
            error!("panic occurred: {:?}", panic_info);
        }
    }));
}

#[tokio::main]
async fn main() {
    init_logging();
    // Don't set panic hook until after logging is set up
    set_panic_hook();

    info!("starting");
    if let Err(e) = os_main().await {
        error!("{:?}", e);

        std::process::exit(1);
    }
}

async fn init_manager() -> Result<Arc<Manager>> {
    let path = std::env::var("DATABASE_URL")
        .with_context(|| "DATABASE_URL environment variable not set")?;
    let db = Database::connect(path, true).await?;
    db.migrate()
        .await
        .with_context(|| "Error migrating database")?;
    let config = Config::try_new()?;
    let manager = Manager::new(&db, &config);
    Ok(Arc::new(manager))
}

enum Transport {
    Http(SocketAddr),
    #[cfg(unix)]
    Uds(String),
}

async fn run_server(
    mut rx: broadcast::Receiver<()>,
    platune_player: Arc<PlatunePlayer>,
    manager: Arc<Manager>,
    transport: Transport,
    is_service: bool,
) -> Result<()> {
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(rpc::FILE_DESCRIPTOR_SET)
        .build()
        .with_context(|| "Error building tonic server")?;

    let player = PlayerImpl::new(platune_player);

    let management = ManagementImpl::new(manager);
    let builder = Server::builder()
        .add_service(reflection_service)
        .add_service(PlayerServer::new(player))
        .add_service(ManagementServer::new(management));

    let server_result = match transport {
        Transport::Http(addr) => {
            builder
                .serve_with_shutdown(addr, async { rx.recv().await.unwrap_or_default() })
                .await
        }
        #[cfg(unix)]
        Transport::Uds(path) => {
            builder
                .serve_with_incoming_shutdown(
                    UnixStream::get_async_stream(&path, !is_service)?,
                    async { rx.recv().await.unwrap_or_default() },
                )
                .await
        }
    };

    server_result.with_context(|| "Error running server")
}

async fn run_servers(tx: broadcast::Sender<()>, is_service: bool) -> Result<()> {
    let platune_player = Arc::new(PlatunePlayer::new());
    let manager = init_manager().await?;

    let mut servers = Vec::<_>::new();
    let http_server = run_server(
        tx.subscribe(),
        platune_player.clone(),
        manager.clone(),
        Transport::Http("0.0.0.0:50051".parse().unwrap()),
        is_service,
    );
    servers.push(http_server);

    #[cfg(unix)]
    {
        let unix_server = run_server(
            tx.subscribe(),
            platune_player,
            manager,
            Transport::Uds("/var/run/platuned/platuned.sock".to_owned()),
            is_service,
        );
        servers.push(unix_server);
    }

    futures::future::try_join_all(servers).await?;
    Ok(())
}

#[cfg(windows)]
async fn os_main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "-i" {
        service::install()?;
    } else if args.len() > 1 && args[1] == "-s" {
        dotenv::from_path(r"C:\Users\asche\code\platune\platuned\server\.env").unwrap();
        service::run()?;
    } else {
        dotenv::from_path("./.env").unwrap();
        run_server(None).await?;
    }

    Ok(())
}

#[cfg(not(windows))]
async fn os_main() -> Result<()> {
    use signal_handler::SignalHandler;

    let args: Vec<String> = std::env::args().collect();

    let mut is_service = true;
    if !(args.len() > 1 && args[1] == "-s") {
        dotenv::from_path("./.env").unwrap();
        is_service = false;
    }
    let (tx, _) = broadcast::channel(32);
    let signal_handler = SignalHandler::start(tx.clone())?;
    run_servers(tx, is_service).await?;
    signal_handler.close().await
}
