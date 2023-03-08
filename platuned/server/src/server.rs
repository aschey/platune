use crate::ipc_stream::IpcStream;
use crate::management_server::ManagementServer;
use crate::player_server::PlayerServer;
use crate::rpc;
use crate::services::management::ManagementImpl;
use crate::services::player::PlayerImpl;
use daemon_slayer::error_handler::color_eyre::eyre::{Context, Result};
use daemon_slayer::server::{BroadcastEventStore, EventStore};
use daemon_slayer::signals::Signal;
use futures::future::try_join_all;
use libplatune_management::config::FileConfig;
use libplatune_management::database::Database;
use libplatune_management::file_watch_manager::FileWatchManager;
use libplatune_management::manager::Manager;
use libplatune_player::platune_player::PlatunePlayer;
use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;
use tonic_reflection::server::Builder;
use tracing::info;

enum Transport {
    Http(SocketAddr),
    Ipc(PathBuf),
}

#[cfg(unix)]
fn create_socket_path(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let parent_dir = path.parent().ok_or_else(|| {
        daemon_slayer::error_handler::color_eyre::eyre::eyre!(
            "Socket path should have a parent directory"
        )
    })?;
    if let Err(e) = std::fs::remove_file(path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            return Err(e).wrap_err("Unable to delete old Unix socket");
        }
    }

    std::fs::create_dir_all(parent_dir).wrap_err("Unable to create Unix socket directory")?;
    let mut perms = parent_dir
        .metadata()
        .wrap_err("Error setting socket directory metadata")?
        .permissions();
    perms.set_mode(0o644);
    Ok(())
}

pub async fn run_all(shutdown_rx: BroadcastEventStore<Signal>) -> Result<()> {
    let platune_player = Arc::new(PlatunePlayer::new(Default::default()));
    let manager = init_manager().await?;
    let manager = FileWatchManager::new(manager, Duration::from_millis(500))
        .await
        .wrap_err("error starting file watch manager")?;

    let mut servers = Vec::<_>::new();
    let http_server = run_server(
        shutdown_rx.clone(),
        platune_player.clone(),
        manager.clone(),
        Transport::Http("[::1]:50051".parse().unwrap()),
    );
    servers.push(http_server);

    #[cfg(unix)]
    let socket_path = {
        let socket_base = match env::var("XDG_RUNTIME_DIR") {
            Ok(socket_base) => socket_base,
            Err(_) => "/tmp".to_owned(),
        };
        let path = Path::new(&socket_base).join("platuned/platuned.sock");
        create_socket_path(&path)?;
        path
    };
    #[cfg(windows)]
    let socket_path = PathBuf::from(r#"\\.\pipe\platuned"#);

    let ipc_server = run_server(
        shutdown_rx.clone(),
        platune_player.clone(),
        manager,
        Transport::Ipc(socket_path),
    );
    servers.push(ipc_server);

    try_join_all(servers).await?;

    let player_inner = Arc::try_unwrap(platune_player).expect("All servers should've been dropped");
    player_inner.join().await?;

    Ok(())
}

async fn init_manager() -> Result<Manager> {
    let path = env::var("DATABASE_URL").wrap_err("DATABASE_URL environment variable not set")?;
    let db = Database::connect(path, true).await?;
    db.sync_database()
        .await
        .wrap_err("Error migrating database")?;
    let config = Arc::new(FileConfig::try_new()?);
    let manager = Manager::new(&db, config);

    Ok(manager)
}

async fn run_server(
    shutdown_rx: BroadcastEventStore<Signal>,
    platune_player: Arc<PlatunePlayer>,
    manager: FileWatchManager,
    transport: Transport,
) -> Result<()> {
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(rpc::FILE_DESCRIPTOR_SET)
        .build()
        .wrap_err("Error building tonic server")?;

    let player = PlayerImpl::new(platune_player, shutdown_rx.clone());

    let management = ManagementImpl::new(manager, shutdown_rx.clone());

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

    health_reporter
        .set_serving::<PlayerServer<PlayerImpl>>()
        .await;
    health_reporter
        .set_serving::<ManagementServer<ManagementImpl>>()
        .await;

    let builder = Server::builder()
        .add_service(reflection_service)
        .add_service(PlayerServer::new(player))
        .add_service(ManagementServer::new(management))
        .add_service(health_service);

    let mut shutdown_rx = shutdown_rx.subscribe_events();
    let server_result = match transport {
        Transport::Http(addr) => {
            info!("Running HTTP server on {addr}");
            builder
                .serve_with_shutdown(addr, async {
                    shutdown_rx.recv().await;
                })
                .await
                .wrap_err("Error running HTTP server")
        }

        Transport::Ipc(path) => {
            info!("Running IPC server on {path:?}");
            builder
                .serve_with_incoming_shutdown(
                    IpcStream::get_async_stream(path.to_string_lossy().to_string())?,
                    async {
                        shutdown_rx.recv().await;
                    },
                )
                .await
                .wrap_err("Error running IPC server")
        }
    };

    server_result.wrap_err("Error running server")
}
