use crate::management_server::ManagementServer;
use crate::player_server::PlayerServer;
use crate::rpc;
use crate::services::management::ManagementImpl;
use crate::services::player::PlayerImpl;
#[cfg(unix)]
use crate::unix::unix_stream::UnixStream;
use anyhow::Context;
use anyhow::Result;
use futures::future::try_join_all;
use libplatune_management::config::Config;
use libplatune_management::database::Database;
use libplatune_management::manager::Manager;
use platune_core::platune_player::PlatunePlayer;
#[cfg(unix)]
use std::env;
use std::env::var;
use std::net::SocketAddr;
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tonic::transport::Server;
use tonic_reflection::server::Builder;
#[cfg(unix)]
use tracing::warn;

enum Transport {
    Http(SocketAddr),
    #[cfg(unix)]
    Uds(PathBuf),
}

pub async fn run_all(shutdown_tx: broadcast::Sender<()>) -> Result<()> {
    let platune_player = Arc::new(PlatunePlayer::default());
    let manager = init_manager().await?;

    let mut servers = Vec::<_>::new();
    let http_server = run_server(
        shutdown_tx.clone(),
        platune_player.clone(),
        manager.clone(),
        Transport::Http("0.0.0.0:50051".parse().unwrap()),
    );
    servers.push(http_server);

    #[cfg(unix)]
    {
        let socket_base = match env::var("XDG_RUNTIME_DIR") {
            Ok(socket_base) => socket_base,
            Err(e) => {
                warn!(
                    "Unable to get XDG_RUNTIME_DIR. Defaulting to /tmp/platuned: {:?}",
                    e
                );
                "/tmp".to_owned()
            }
        };
        let socket_path = Path::new(&socket_base).join("platuned/platuned.sock");
        let unix_server = run_server(
            shutdown_tx.clone(),
            platune_player,
            manager,
            Transport::Uds(socket_path),
        );
        servers.push(unix_server);
    }

    try_join_all(servers).await?;
    Ok(())
}

async fn init_manager() -> Result<Arc<RwLock<Manager>>> {
    let path = var("DATABASE_URL").with_context(|| "DATABASE_URL environment variable not set")?;
    let db = Database::connect(path, true).await?;
    db.migrate()
        .await
        .with_context(|| "Error migrating database")?;
    let config = Config::try_new()?;
    let manager = Manager::new(&db, &config);

    Ok(Arc::new(RwLock::new(manager)))
}

async fn run_server(
    shutdown_tx: broadcast::Sender<()>,
    platune_player: Arc<PlatunePlayer>,
    manager: Arc<RwLock<Manager>>,
    transport: Transport,
) -> Result<()> {
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(rpc::FILE_DESCRIPTOR_SET)
        .build()
        .with_context(|| "Error building tonic server")?;

    let player = PlayerImpl::new(platune_player, shutdown_tx.clone());

    let management = ManagementImpl::new(manager, shutdown_tx.clone());
    let builder = Server::builder()
        .add_service(reflection_service)
        .add_service(PlayerServer::new(player))
        .add_service(ManagementServer::new(management));

    let mut shutdown_rx = shutdown_tx.subscribe();
    let server_result = match transport {
        Transport::Http(addr) => builder
            .serve_with_shutdown(addr, async { shutdown_rx.recv().await.unwrap_or_default() })
            .await
            .with_context(|| "Error running HTTP server"),
        #[cfg(unix)]
        Transport::Uds(path) => builder
            .serve_with_incoming_shutdown(UnixStream::get_async_stream(&path)?, async {
                shutdown_rx.recv().await.unwrap_or_default()
            })
            .await
            .with_context(|| "Error running UDS server"),
    };

    server_result.with_context(|| "Error running server")
}
