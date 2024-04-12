use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
#[cfg(feature = "management")]
use std::time::Duration;

use daemon_slayer::error_handler::color_eyre::eyre::{Context, Result};
use daemon_slayer::server::{BroadcastEventStore, EventStore, Signal};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
#[cfg(feature = "management")]
use libplatune_management::config::config_dir;
#[cfg(feature = "management")]
use libplatune_management::config::FileConfig;
#[cfg(feature = "management")]
use libplatune_management::database::Database;
#[cfg(feature = "management")]
use libplatune_management::file_watch_manager::FileWatchManager;
#[cfg(feature = "management")]
use libplatune_management::manager::Manager;
#[cfg(feature = "player")]
use libplatune_player::platune_player::PlatunePlayer;
#[cfg(feature = "player")]
use libplatune_player::CpalOutput;
use platuned::{file_server_port, main_server_port};
use tonic::transport::Server;
use tonic_reflection::server::Builder;
#[cfg(feature = "management")]
use tower_http::services::ServeDir;
use tracing::info;

use crate::cert_gen::{get_tls_config, get_tonic_tls_config};
use crate::ipc_stream::IpcStream;
#[cfg(feature = "management")]
use crate::management_server::ManagementServer;
#[cfg(feature = "player")]
use crate::player_server::PlayerServer;
use crate::rpc;
#[cfg(feature = "management")]
use crate::services::management::ManagementImpl;
#[cfg(feature = "player")]
use crate::services::player::PlayerImpl;

enum Transport {
    Http(SocketAddr),
    Ipc(&'static str),
}

#[derive(Clone)]
struct Services {
    #[cfg(feature = "player")]
    player: Arc<PlatunePlayer<CpalOutput>>,
    #[cfg(feature = "management")]
    manager: FileWatchManager,
}

impl Services {
    async fn new() -> Result<Self> {
        #[cfg(feature = "management")]
        let manager = init_manager().await?;
        Ok(Self {
            #[cfg(feature = "player")]
            player: Arc::new(PlatunePlayer::new(Default::default(), Default::default())),
            #[cfg(feature = "management")]
            manager: FileWatchManager::new(manager, Duration::from_millis(500))
                .await
                .wrap_err("error starting file watch manager")?,
        })
    }
}

#[cfg(not(feature = "management"))]
pub fn config_dir() -> Result<PathBuf> {
    let proj_dirs =
        directories::ProjectDirs::from("", "", "platune").ok_or_else(|| eyre!("No home dir"))?;
    Ok(proj_dirs.config_dir().to_path_buf())
}

pub async fn run_all(shutdown_rx: BroadcastEventStore<Signal>) -> Result<()> {
    let services = Services::new().await?;
    let servers = FuturesUnordered::new();
    let port = main_server_port()?;
    let http_server = run_server(
        shutdown_rx.clone(),
        services.clone(),
        Transport::Http(
            format!("0.0.0.0:{port}")
                .parse()
                .expect("failed to parse address"),
        ),
    );
    servers.push(tokio::spawn(http_server));

    let ipc_server = run_server(
        shutdown_rx.clone(),
        services.clone(),
        Transport::Ipc("platune/platuned"),
    );
    servers.push(tokio::spawn(ipc_server));
    #[cfg(feature = "management")]
    {
        let folders = services.manager.read().await.get_all_folders().await?;
        if !folders.is_empty() {
            servers.push(tokio::spawn(run_file_service(folders, shutdown_rx)));
        }
    }

    let _: Vec<_> = servers.collect().await;

    #[cfg(feature = "player")]
    {
        let player_inner =
            Arc::try_unwrap(services.player).expect("All servers should've been dropped");
        player_inner.join().await?;
    }

    Ok(())
}

#[cfg(feature = "management")]
async fn run_file_service(
    folders: Vec<String>,
    shutdown_rx: BroadcastEventStore<Signal>,
) -> Result<()> {
    let addr: SocketAddr = format!("0.0.0.0:{}", file_server_port()?)
        .parse()
        .expect("failed to parse address");
    let mut shutdown_rx = shutdown_rx.subscribe_events();
    info!("Running file server on {addr}");
    let mut app = axum::Router::new();
    let root_path = "/";
    match &folders[..] {
        [] => {}
        [folder] => {
            app = app.nest_service(root_path, ServeDir::new(folder));
        }
        [folder, fallback] => {
            app = app.nest_service(
                root_path,
                ServeDir::new(folder).fallback(ServeDir::new(fallback)),
            );
        }
        [first, second, rest @ ..] => {
            let mut serve_dir = ServeDir::new(first).fallback(ServeDir::new(second));
            for folder in rest {
                serve_dir = serve_dir.fallback(ServeDir::new(folder));
            }
            app = app.nest_service(root_path, serve_dir);
        }
    }

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .wrap_err(format!("Failed to bind to {addr}"))?;
    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        shutdown_rx.next().await;
    });
    server.await.wrap_err("Error running file server")?;

    Ok(())
}

#[cfg(feature = "management")]
async fn init_manager() -> Result<Manager> {
    let path = env::var("DATABASE_URL")
        .wrap_err("DATABASE_URL environment variable not set")?
        .replace("sqlite://", "");

    info!("Connecting to database {path:?}");
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
    services: Services,
    transport: Transport,
) -> Result<()> {
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(rpc::FILE_DESCRIPTOR_SET)
        .build()
        .wrap_err("Error building tonic server")?;

    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();

    #[cfg(feature = "player")]
    health_reporter
        .set_serving::<PlayerServer<PlayerImpl>>()
        .await;

    #[cfg(feature = "management")]
    health_reporter
        .set_serving::<ManagementServer<ManagementImpl>>()
        .await;

    let mut builder = Server::builder();
    if matches!(transport, Transport::Http(_))
        && matches!(
            std::env::var("PLATUNE_ENABLE_TLS").as_deref(),
            Ok("1" | "true")
        )
    {
        let config_dir = config_dir()?;
        let server_tls = get_tls_config(&config_dir.join("server")).await?;
        let client_tls = if matches!(
            std::env::var("PLATUNE_ENABLE_CLIENT_TLS").as_deref(),
            Ok("1" | "true")
        ) {
            Some(get_tls_config(&config_dir.join("client")).await?)
        } else {
            None
        };
        let server_tls_config = get_tonic_tls_config(server_tls, client_tls);
        builder = builder.tls_config(server_tls_config)?;
    }

    let builder = builder
        .add_service(reflection_service)
        .add_service(health_service);
    #[cfg(feature = "player")]
    let builder = builder.add_service(PlayerServer::new(PlayerImpl::new(
        services.player,
        shutdown_rx.clone(),
    )));
    #[cfg(feature = "management")]
    let builder = builder.add_service(ManagementServer::new(ManagementImpl::new(
        services.manager,
        shutdown_rx.clone(),
    )));

    let mut shutdown_rx = shutdown_rx.subscribe_events();
    let server_result = match transport {
        Transport::Http(addr) => {
            info!("Running HTTP server on {addr}");
            builder
                .serve_with_shutdown(addr, async {
                    shutdown_rx.next().await;
                })
                .await
                .wrap_err("Error running HTTP server")
        }

        Transport::Ipc(path) => {
            info!("Running IPC server on {path:?}");
            builder
                .serve_with_incoming_shutdown(IpcStream::get_async_stream(path)?, async {
                    shutdown_rx.next().await;
                })
                .await
                .wrap_err("Error running IPC server")
        }
    };

    server_result.wrap_err("Error running server")
}
