use crate::management::ManagementImpl;
use crate::management_server::ManagementServer;
use crate::player::PlayerImpl;
use crate::player_server::PlayerServer;
use crate::rpc;
#[cfg(unix)]
use crate::unix::unix_stream::UnixStream;
use anyhow::Context;
use anyhow::Result;
use libplatune_management::config::Config;
use libplatune_management::database::Database;
use libplatune_management::manager::Manager;
use libplatune_player::platune_player::PlatunePlayer;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tonic::transport::Server;

enum Transport {
    Http(SocketAddr),
    #[cfg(unix)]
    Uds(String),
}

pub async fn run_all(shutdown_tx: broadcast::Sender<()>) -> Result<()> {
    let platune_player = Arc::new(PlatunePlayer::new());
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
        let unix_server = run_server(
            shutdown_tx.clone(),
            platune_player,
            manager,
            Transport::Uds("/var/run/platuned/platuned.sock".to_owned()),
        );
        servers.push(unix_server);
    }

    futures::future::try_join_all(servers).await?;
    Ok(())
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

async fn run_server(
    shutdown_tx: broadcast::Sender<()>,
    platune_player: Arc<PlatunePlayer>,
    manager: Arc<Manager>,
    transport: Transport,
) -> Result<()> {
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(rpc::FILE_DESCRIPTOR_SET)
        .build()
        .with_context(|| "Error building tonic server")?;

    let player = PlayerImpl::new(platune_player);

    let management = ManagementImpl::new(manager, shutdown_tx.clone());
    let builder = Server::builder()
        .add_service(reflection_service)
        .add_service(PlayerServer::new(player))
        .add_service(ManagementServer::new(management));

    let mut shutdown_rx = shutdown_tx.subscribe();
    let server_result = match transport {
        Transport::Http(addr) => {
            builder
                .serve_with_shutdown(addr, async { shutdown_rx.recv().await.unwrap_or_default() })
                .await
        }
        #[cfg(unix)]
        Transport::Uds(path) => {
            builder
                .serve_with_incoming_shutdown(UnixStream::get_async_stream(&path)?, async {
                    shutdown_rx.recv().await.unwrap_or_default()
                })
                .await
        }
    };

    server_result.with_context(|| "Error running server")
}
