mod management;
mod player;
use crate::management_server::ManagementServer;
use crate::player_server::PlayerServer;
use flexi_logger::{style, DeferredNow, Logger, Record};
use management::ManagementImpl;
use player::PlayerImpl;
use rpc::*;

use tokio::sync::mpsc::Receiver;
use tonic::transport::Server;
use yansi::{Color, Style};

pub mod rpc {
    tonic::include_proto!("player_rpc");
    tonic::include_proto!("management_rpc");

    pub(crate) const FILE_DESCRIPTOR_SET: &'static [u8] =
        tonic::include_file_descriptor_set!("rpc_descriptor");
}

#[cfg(windows)]
mod service {
    use std::{
        env,
        ffi::{OsStr, OsString},
        time::Duration,
    };
    use tokio::runtime::Runtime;
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

    pub fn run() {
        service_dispatcher::start(SERVICE_NAME, service_main).unwrap();
    }

    pub fn install() {
        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access).unwrap();
        let service_access =
            ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
        if let Ok(service) = service_manager.open_service(SERVICE_NAME, service_access) {
            if service.query_status().unwrap().current_state == ServiceState::Running {
                service.stop().unwrap();
            }
            service.delete().unwrap();
        }

        // This example installs the service defined in `examples/ping_service.rs`.
        // In the real world code you would set the executable path to point to your own binary
        // that implements windows service.
        let service_binary_path = ::std::env::current_exe().unwrap();

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
        let service = service_manager
            .create_service(
                &service_info,
                ServiceAccess::CHANGE_CONFIG | ServiceAccess::START,
            )
            .unwrap();
        service.set_description("platune service").unwrap();
        service.start(&[OsStr::new("Started")]).unwrap();
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
                    event_tx.try_send(()).unwrap();
                    ServiceControlHandlerResult::NoError
                }

                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        // Register system service event handler.
        // The returned status handle should be used to report service status changes to the system.
        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler).unwrap();

        // Tell the system that service is running
        status_handle
            .set_service_status(ServiceStatus {
                service_type: SERVICE_TYPE,
                current_state: ServiceState::Running,
                controls_accepted: ServiceControlAccept::STOP,
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })
            .unwrap();

        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            run_server(Some(event_rx)).await;
        });

        // Tell the system that service has stopped.
        status_handle
            .set_service_status(ServiceStatus {
                service_type: SERVICE_TYPE,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::Win32(0),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })
            .unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Logger::try_with_str("info")
        .unwrap()
        .format_for_stdout(colored)
        .log_to_stdout()
        .set_palette("196;190;-;-;-".to_owned())
        .start()
        .unwrap();

    os_main().await;

    Ok(())
}

async fn run_server(rx: Option<Receiver<()>>) {
    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(rpc::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let addr = "0.0.0.0:50051".parse().unwrap();

    let player = PlayerImpl::new();
    let management = ManagementImpl::new().await;
    let builder = Server::builder()
        .add_service(service)
        .add_service(PlayerServer::new(player))
        .add_service(ManagementServer::new(management));
    match rx {
        Some(mut rx) => builder
            .serve_with_shutdown(addr, async { rx.recv().await.unwrap_or_default() })
            .await
            .unwrap(),
        None => builder.serve(addr).await.unwrap(),
    }
}

#[cfg(windows)]
async fn os_main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "-i" {
        service::install();
        return;
    }

    if args.len() > 1 && args[1] == "-s" {
        dotenv::from_path(r"C:\Users\asche\code\platune\platuned\server\.env").unwrap();
        service::run();
        return;
    }
    dotenv::from_path("./.env").unwrap();
    run_server(None).await;
}

#[cfg(not(windows))]
async fn os_main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "-s" {
        run_server(None).await;
        return;
    }
    dotenv::from_path("./.env").unwrap();
    run_server(None).await;
}

pub fn colored(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> core::result::Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "[{} {}] {} [{}:{}] {}",
        Style::new(Color::Cyan).paint(now.now().format("%Y-%m-%d %H:%M:%S%.6f")),
        Style::new(Color::RGB(119, 102, 204)).paint(now.now().timestamp_nanos() as f64 / 1e9),
        style(level).paint(format!("{}", level)),
        Style::new(Color::Green).paint(record.file().unwrap_or_else(|| "<unnamed>")),
        Style::new(Color::Green).paint(record.line().unwrap_or_else(|| 0)),
        style(level).paint(format!("{}", record.args()))
    )
}
