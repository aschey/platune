use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use background_service::ServiceContext;
use daemon_slayer::core::notify::AsyncNotification;
use daemon_slayer::error_handler::color_eyre::eyre::{Context, Result};
use daemon_slayer::notify::notification::Notification;
use daemon_slayer::server::{BroadcastEventStore, EventStore, Signal};
use futures::StreamExt;
#[cfg(feature = "management")]
use libplatune_management::config::FileConfig;
#[cfg(feature = "management")]
use libplatune_management::config::config_dir;
#[cfg(feature = "management")]
use libplatune_management::database::Database;
#[cfg(feature = "management")]
use libplatune_management::file_watch_manager::FileWatchManager;
#[cfg(feature = "management")]
use libplatune_management::manager::Manager;
#[cfg(feature = "player")]
use libplatune_player::CpalOutput;
#[cfg(feature = "player")]
use libplatune_player::platune_player::PlatunePlayer;
#[cfg(feature = "player")]
use libplatune_player::platune_player::PlayerEvent;
use platuned::{file_server_port, main_server_port, service_label};
use tipsy::{IntoIpcPath, ServerId};
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tonic_reflection::server::Builder;
#[cfg(feature = "management")]
use tower_http::services::ServeDir;
use tracing::{info, warn};

use crate::cert_gen::{get_tls_config, get_tonic_tls_config};
use crate::ipc_stream::IpcStream;
use crate::rpc;
#[cfg(feature = "management")]
use crate::services::management::ManagementImpl;
#[cfg(feature = "player")]
use crate::services::player::PlayerImpl;
#[cfg(feature = "management")]
use crate::v1::management_server::ManagementServer;
#[cfg(feature = "player")]
use crate::v1::player_server::PlayerServer;

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
            manager: FileWatchManager::new(manager, Duration::from_millis(500), move || {
                Box::pin(async move {
                    let _ = Notification::new(service_label())
                        .summary("Sync completed")
                        .show()
                        .await
                        .inspect_err(|e| warn!("Error sending notification: {e:?}"));
                })
            })
            .await
            .wrap_err("error starting file watch manager")?,
        })
    }
}

#[cfg(not(feature = "management"))]
pub fn config_dir() -> Result<std::path::PathBuf> {
    use daemon_slayer::error_handler::color_eyre::eyre::eyre;
    let proj_dirs =
        directories::ProjectDirs::from("", "", "platune").ok_or_else(|| eyre!("No home dir"))?;
    Ok(proj_dirs.config_dir().to_path_buf())
}

pub async fn run_all(shutdown_rx: BroadcastEventStore<Signal>) -> Result<()> {
    let services = Services::new().await?;
    let port = main_server_port()?;

    #[cfg(feature = "player")]
    show_notifications(&services.player);

    let manager = background_service::Manager::new(
        CancellationToken::new(),
        background_service::Settings::default(),
    );
    let context = manager.get_context();
    tokio::spawn({
        let mut shutdown_rx = shutdown_rx.subscribe_events();
        let context = context.clone();
        async move {
            shutdown_rx.next().await;
            context.cancellation_token().cancel();
        }
    });

    context.spawn(("http_server", {
        let services = services.clone();
        move |context: ServiceContext| async move {
            run_server(
                services,
                Transport::Http(
                    format!("0.0.0.0:{port}")
                        .parse()
                        .expect("failed to parse address"),
                ),
                context.cancellation_token().clone(),
            )
            .await?;
            Ok(())
        }
    }));

    context.spawn(("ipc_server", {
        let services = services.clone();
        |context: ServiceContext| async move {
            run_server(
                services,
                Transport::Ipc("platune/platuned"),
                context.cancellation_token().clone(),
            )
            .await?;
            Ok(())
        }
    }));
    #[cfg(feature = "management")]
    {
        let folders = services.manager.read().await.get_all_folders().await?;
        if !folders.is_empty() {
            context.spawn(("file_service", |context: ServiceContext| async move {
                run_file_service(folders, context.cancellation_token().clone()).await?;
                Ok(())
            }));
        }
    }

    let _ = manager
        .join_on_cancel()
        .await
        .inspect_err(|e| warn!("{e:?}"));
    info!("All servers terminated");

    #[cfg(feature = "player")]
    {
        let player = Arc::try_unwrap(services.player).expect("servers not dropped");
        player.join().await?;
    }

    Ok(())
}

#[cfg(feature = "player")]
fn show_notifications(player: &Arc<PlatunePlayer<CpalOutput>>) {
    let mut player_rx = player.subscribe();
    tokio::spawn(async move {
        while let Ok(event) = player_rx.recv().await {
            if let PlayerEvent::TrackChanged(state) = event
                && let Some(meta) = state.metadata
            {
                let msg = [meta.song, meta.artist]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" - ");
                let _ = Notification::new(service_label())
                    .summary(format!("Now playing: {msg}"))
                    .show()
                    .await
                    .inspect_err(|e| warn!("Error sending notification: {e:?}"));
            }
        }
    });
}

#[cfg(feature = "management")]
async fn run_file_service(
    folders: Vec<String>,
    cancellation_token: CancellationToken,
) -> Result<()> {
    let addr: SocketAddr = format!("0.0.0.0:{}", file_server_port()?)
        .parse()
        .expect("failed to parse address");
    info!("Running file server on {addr}");
    let mut app = axum::Router::new();
    let root_path = "/";
    match &folders[..] {
        [] => {}
        [folder] => {
            app = app.fallback_service(ServeDir::new(folder));
        }
        [folder, fallback] => {
            app = app.fallback_service(ServeDir::new(folder).fallback(ServeDir::new(fallback)));
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
        cancellation_token.cancelled().await;
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
    services: Services,
    transport: Transport,
    cancellation_token: CancellationToken,
) -> Result<()> {
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(rpc::FILE_DESCRIPTOR_SET)
        .build_v1()
        .wrap_err("Error building tonic server")?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();

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
        info!("Enabling TLS");
        let config_dir = config_dir()?;
        let server_tls = get_tls_config(&config_dir.join("server")).await?;
        let client_tls = if matches!(
            std::env::var("PLATUNE_ENABLE_CLIENT_TLS").as_deref(),
            Ok("1" | "true")
        ) {
            info!("Enabling client TLS");
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
        cancellation_token.clone(),
    )));
    #[cfg(feature = "management")]
    let builder = builder.add_service(ManagementServer::new(ManagementImpl::new(
        services.manager,
        cancellation_token.clone(),
    )));

    let server_result = match transport {
        Transport::Http(addr) => {
            info!("Running HTTP server on {addr}");

            builder
                .serve_with_shutdown(addr, cancellation_token.cancelled())
                .await
                .wrap_err("Error running HTTP server")
        }

        Transport::Ipc(path) => {
            let ipc_path = ServerId::new(path).parent_folder("/tmp").into_ipc_path()?;
            info!("Running IPC server on {}", ipc_path.display());

            builder
                .serve_with_incoming_shutdown(
                    IpcStream::get_async_stream(ipc_path)?,
                    cancellation_token.cancelled(),
                )
                .await
                .wrap_err("Error running IPC server")
        }
    };

    server_result.wrap_err("Error running server")
}
