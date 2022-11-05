use crate::management_server::ManagementServer;
use crate::player_server::PlayerServer;
use crate::rpc::*;
use crate::services::management::ManagementImpl;
use crate::services::player::PlayerImpl;
use crate::sync_handler_client::SyncHandlerClient;
#[cfg(unix)]
use crate::unix::unix_stream::UnixStream;
use daemon_slayer::error_handler::color_eyre::eyre::{Context, Result};
use daemon_slayer::server::{BroadcastEventStore, EventStore};
use daemon_slayer::signals::Signal;
use futures::future::try_join_all;
use futures::StreamExt;
use libplatune_management::file_watcher::file_watch_manager::FileWatchManager;
use libplatune_player::platune_player::PlatunePlayer;
#[cfg(unix)]
use std::env;
use std::net::SocketAddr;
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::path::PathBuf;
use std::sync::Arc;
use tonic::transport::Server;
use tonic_reflection::server::Builder;
#[cfg(unix)]
use tracing::warn;

enum Transport {
    Http(SocketAddr),
    #[cfg(unix)]
    Uds(PathBuf),
}

pub async fn run_all(
    manager: FileWatchManager,
    sync_client: SyncHandlerClient,
    progress_store: BroadcastEventStore<Progress>,
    shutdown_rx: BroadcastEventStore<Signal>,
) -> Result<()> {
    let platune_player = Arc::new(PlatunePlayer::new(Default::default()));

    let mut servers = Vec::<_>::new();
    let http_server = run_server(
        shutdown_rx.clone(),
        platune_player.clone(),
        manager.clone(),
        sync_client.clone(),
        progress_store.clone(),
        Transport::Http("[::1]:50051".parse().unwrap()),
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
            shutdown_rx.clone(),
            platune_player.clone(),
            manager,
            sync_client,
            progress_store,
            Transport::Uds(socket_path),
        );
        servers.push(unix_server);
    }

    try_join_all(servers).await?;

    let player_inner = Arc::try_unwrap(platune_player).expect("All servers should've been dropped");
    player_inner.join().await?;

    Ok(())
}

async fn run_server(
    shutdown_rx: BroadcastEventStore<Signal>,
    platune_player: Arc<PlatunePlayer>,
    manager: FileWatchManager,
    sync_client: SyncHandlerClient,
    progress_store: BroadcastEventStore<Progress>,
    transport: Transport,
) -> Result<()> {
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .wrap_err("Error building tonic server")?;

    let player = PlayerImpl::new(platune_player, shutdown_rx.clone());

    let management = ManagementImpl::new(manager, sync_client, progress_store, shutdown_rx.clone());

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
        Transport::Http(addr) => builder
            .serve_with_shutdown(addr, async {
                shutdown_rx.next().await;
            })
            .await
            .wrap_err("Error running HTTP server"),
        #[cfg(unix)]
        Transport::Uds(path) => builder
            .serve_with_incoming_shutdown(UnixStream::get_async_stream(&path)?, async {
                shutdown_rx.next().await;
            })
            .await
            .wrap_err("Error running UDS server"),
    };

    server_result.wrap_err("Error running server")
}
