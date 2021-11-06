use crate::server;
use anyhow::{Context, Result};
use std::{
    env::current_exe,
    ffi::{OsStr, OsString},
    time::Duration,
};
use tokio::runtime::Runtime;
use tracing::{error, info};
use windows_service::{
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
    service_manager::{ServiceManager, ServiceManagerAccess},
};

windows_service::define_windows_service!(service_main, handle_service_main);

const SERVICE_NAME: &str = "platuned";
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

pub fn run() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, service_main).with_context(|| "Error starting service")
}

pub fn install() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)
        .with_context(|| "Error connecting to service database")?;
    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    if let Ok(service) = service_manager.open_service(SERVICE_NAME, service_access) {
        let status = service
            .query_status()
            .with_context(|| "Error querying service status")?;
        if status.current_state == ServiceState::Running {
            service.stop().with_context(|| "Error stopping service")?;
        }
        service.delete().with_context(|| "Error deleting service")?;
    }

    let service_binary_path = current_exe().with_context(|| "Error getting current exe path")?;

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
    let (event_tx, _) = tokio::sync::broadcast::channel(32);

    // Define system service event handler that will be receiving service events.
    let event_tx_ = event_tx.clone();
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NoError even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            // Handle stop
            ServiceControl::Stop => {
                info!("Sending shutdown signal");
                if let Err(e) = event_tx_.send(()) {
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

    let exit_code = match rt.block_on(async { server::run_all(event_tx).await }) {
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
