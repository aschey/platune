use std::env;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::time::Duration;

use daemon_slayer::server::{BroadcastEventStore, EventStore, Signal};
use futures::{Stream, StreamExt};
use libplatune_management::file_watch_manager::FileWatchManager;
use libplatune_management::manager::{Manager, SearchOptions};
use libplatune_management::{database, manager};
use platuned::file_server_port;
use prost_types::Timestamp;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{RwLockReadGuard, mpsc};
use tonic::{Request, Response, Status, Streaming};
use tracing::{error, info, warn};

use crate::rpc::v1::*;
use crate::v1::management_server::Management;

pub struct ManagementImpl {
    manager: FileWatchManager,
    shutdown_rx: BroadcastEventStore<Signal>,
}

impl ManagementImpl {
    pub(crate) fn new(manager: FileWatchManager, shutdown_rx: BroadcastEventStore<Signal>) -> Self {
        Self {
            manager,
            shutdown_rx,
        }
    }
}

fn format_error(msg: String) -> Status {
    error!("{:?}", msg);
    Status::internal(msg)
}

enum ConnectionType {
    Local,
    Remote {
        folders: Vec<String>,
        local_addr: String,
    },
}

#[allow(clippy::result_large_err)]
fn map_lookup_entry(
    entry: database::LookupEntry,
    connection_type: &ConnectionType,
) -> Result<LookupEntry, Status> {
    let path = match connection_type {
        ConnectionType::Local => format!("file://{}", entry.path),
        ConnectionType::Remote {
            folders,
            local_addr,
        } => {
            let folder = match folders.iter().find(|f| entry.path.starts_with(*f)) {
                Some(folder) => folder,
                None => {
                    return Err(format_error(format!(
                        "Unable to find folder for path {}",
                        entry.path
                    )));
                }
            };
            entry.path.replacen(folder, local_addr, 1)
        }
    };

    Ok(LookupEntry {
        artist: entry.artist,
        album_artist: entry.album_artist,
        album: entry.album,
        song: entry.song,
        path,
        track: entry.track,
        duration: Some(Timestamp {
            seconds: Duration::from_millis(entry.duration_millis as u64).as_secs() as i64,
            nanos: Duration::from_millis(entry.duration_millis as u64).subsec_nanos() as i32,
        }),
    })
}

async fn get_connection_type<T>(
    request: &Request<T>,
    manager: &RwLockReadGuard<'_, Manager>,
) -> Result<ConnectionType, Status> {
    let remote_addr = if let Some(addr) = request.remote_addr() {
        if let Ok(header) = env::var("PLATUNE_IP_HEADER") {
            let ip = request
                .metadata()
                .get(&header)
                .map(|ip| ip.to_str().unwrap().parse::<IpAddr>());
            info!("Using custom header for source IP {header}: {ip:?}");
            if let Some(Ok(ip)) = ip { ip } else { addr.ip() }
        } else {
            addr.ip()
        }
    } else {
        return Ok(ConnectionType::Local);
    };
    info!("Remote addr {remote_addr:?}");
    let is_remote = !remote_addr.is_loopback();

    if is_remote {
        let folders = manager
            .get_all_folders()
            .await
            .map_err(|e| format_error(format!("Error getting folders: {e:?}")))?
            .into_iter()
            .map(|f| {
                if cfg!(windows) {
                    f.replace('\\', "/")
                } else {
                    f
                }
            })
            .collect();

        if !is_local(remote_addr) {
            if let Ok(mut global_addr) = env::var("PLATUNE_GLOBAL_FILE_URL") {
                if !global_addr.ends_with('/') {
                    global_addr.push('/');
                }
                info!("Using global file URL {global_addr}");
                return Ok(ConnectionType::Remote {
                    folders,
                    local_addr: global_addr,
                });
            }
        }
        let local_addr = match request
            .local_addr()
            .ok_or_else(|| format_error("Local address missing".to_string()))?
        {
            SocketAddr::V4(addr) => addr.ip().to_string(),
            SocketAddr::V6(addr) => format!("[{}]", addr.ip()),
        };
        Ok(ConnectionType::Remote {
            folders,
            local_addr: format!("http://{local_addr}:{}/", file_server_port().unwrap()),
        })
    } else {
        Ok(ConnectionType::Local)
    }
}

fn is_local(ip_addr: IpAddr) -> bool {
    ip_addr.is_loopback()
        || ip_addr.is_unspecified()
        || match ip_addr {
            IpAddr::V4(v4) => v4.is_link_local() || v4.is_private(),
            IpAddr::V6(v6) => v6.is_unicast_link_local() || v6.is_unique_local(),
        }
}

#[tonic::async_trait]
impl Management for ManagementImpl {
    async fn start_sync(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.manager.start_sync_all().await {
            Ok(_) => Ok(Response::new(())),
            Err(e) => Err(format_error(e.to_string())),
        }
    }

    type SubscribeEventsStream =
        Pin<Box<dyn Stream<Item = Result<Progress, Status>> + Send + Sync + 'static>>;

    async fn subscribe_events(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
        let mut progress_rx = self.manager.subscribe_progress();
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(async move {
            loop {
                match progress_rx.recv().await {
                    Ok(val) => {
                        tx.send(Ok(Progress {
                            job: val.job,
                            percentage: val.percentage,
                            finished: val.finished,
                        }))
                        .await
                        .unwrap_or_default();
                    }
                    Err(RecvError::Lagged(_)) => {}
                    _ => {
                        break;
                    }
                }
            }
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }

    async fn add_folders(&self, request: Request<FoldersMessage>) -> Result<Response<()>, Status> {
        if let Err(e) = self
            .manager
            .write()
            .await
            .add_folders(
                request
                    .into_inner()
                    .folders
                    .iter()
                    .map(|s| &s[..])
                    .collect(),
            )
            .await
        {
            return Err(format_error(format!("Error adding folders {e:?}")));
        };
        Ok(Response::new(()))
    }

    async fn get_all_folders(&self, _: Request<()>) -> Result<Response<FoldersMessage>, Status> {
        let folders = match self.manager.read().await.get_all_folders().await {
            Ok(f) => f,
            Err(e) => {
                return Err(format_error(format!("Error syncing files {e:?}")));
            }
        };
        Ok(Response::new(FoldersMessage { folders }))
    }

    async fn register_mount(
        &self,
        request: Request<RegisteredMountMessage>,
    ) -> Result<Response<()>, Status> {
        match self
            .manager
            .write()
            .await
            .register_drive(&request.into_inner().mount)
            .await
        {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(Status::invalid_argument(format!("{e}"))),
        }
    }

    async fn get_registered_mount(
        &self,
        _: Request<()>,
    ) -> Result<Response<RegisteredMountMessage>, Status> {
        let mount = self.manager.read().await.get_registered_mount().await;
        Ok(Response::new(RegisteredMountMessage {
            mount: mount.unwrap_or_default(),
        }))
    }

    async fn get_albums_by_album_artists(
        &self,
        request: Request<IdMessage>,
    ) -> Result<Response<AlbumResponse>, Status> {
        let request = request.into_inner();
        let albums = self
            .manager
            .read()
            .await
            .albums_by_album_artists(request.ids)
            .await
            .map_err(|e| format_error(format!("Error getting albums: {e:?}")))?;
        Ok(Response::new(AlbumResponse {
            entries: albums
                .into_iter()
                .map(|a| AlbumEntry {
                    album: a.album,
                    album_id: a.album_id,
                    album_artist: a.album_artist,
                    album_artist_id: a.album_artist_id,
                })
                .collect(),
        }))
    }

    async fn lookup(
        &self,
        request: Request<LookupRequest>,
    ) -> Result<Response<LookupResponse>, Status> {
        let manager = self.manager.read().await;
        let connection_type = get_connection_type(&request, &manager).await?;
        let request = request.into_inner();
        let lookup_result = match manager
            .lookup(
                request.correlation_ids,
                match EntryType::try_from(request.entry_type).unwrap() {
                    EntryType::Song => manager::EntryType::Song,
                    EntryType::Album => manager::EntryType::Album,
                    EntryType::Artist => manager::EntryType::Artist,
                },
            )
            .await
        {
            Ok(entries) => entries,
            Err(e) => {
                return Err(format_error(format!("Error sending lookup request {e:?}")));
            }
        };

        let entries: Result<Vec<_>, _> = lookup_result
            .into_iter()
            .map(|e| map_lookup_entry(e, &connection_type))
            .collect();

        Ok(Response::new(LookupResponse { entries: entries? }))
    }

    type SearchStream = Pin<
        Box<dyn futures::Stream<Item = Result<SearchResponse, Status>> + Send + Sync + 'static>,
    >;

    async fn search(
        &self,
        request: Request<Streaming<SearchRequest>>,
    ) -> Result<Response<Self::SearchStream>, Status> {
        let mut messages = request.into_inner();
        let manager = self.manager.clone();

        let (tx, rx) = mpsc::channel(32);
        // Close stream when shutdown is requested
        let mut shutdown_rx = self.shutdown_rx.subscribe_events();

        tokio::spawn(async move {
            while let Some(msg) =
                tokio::select! { val = messages.next() => val, _ = shutdown_rx.next() => None }
            {
                let manager = manager.read().await;
                match msg {
                    Ok(msg) => {
                        let options = SearchOptions {
                            ..Default::default()
                        };
                        if let Err(e) = tx.send(manager.search(&msg.query, options).await).await {
                            warn!("Error sending message to response stream {:?}", e);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Response::new(Box::pin({
            tokio_stream::wrappers::ReceiverStream::new(rx).map(|r| {
                let search_results = match r {
                    Ok(results) => results,
                    Err(e) => {
                        return Err(format_error(format!("Error sending search request {e:?}")));
                    }
                };
                let results = search_results
                    .into_iter()
                    .map(|res| SearchResult {
                        description: res.description,
                        entry: res.entry,
                        entry_type: (match res.entry_type {
                            manager::EntryType::Song => EntryType::Song,
                            manager::EntryType::Artist => EntryType::Artist,
                            manager::EntryType::Album => EntryType::Album,
                        })
                        .into(),
                        artist: res.artist,
                        correlation_ids: res.correlation_ids,
                    })
                    .collect();
                Ok(SearchResponse { results })
            })
        })))
    }

    async fn get_deleted(&self, _: Request<()>) -> Result<Response<GetDeletedResponse>, Status> {
        let deleted_songs = match self.manager.read().await.get_deleted_songs().await {
            Ok(songs) => songs,
            Err(e) => return Err(format_error(format!("Error getting deleted songs {e:?}"))),
        };
        return Ok(Response::new(GetDeletedResponse {
            results: deleted_songs
                .into_iter()
                .map(|d| DeletedResult {
                    path: d.song_path,
                    id: d.song_id,
                })
                .collect(),
        }));
    }

    async fn delete_tracks(&self, request: Request<IdMessage>) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        let manager = self.manager.write().await;
        if let Err(e) = manager.delete_tracks(request.ids).await {
            return Err(format_error(format!("Error deleting tracks {e:?}")));
        }

        Ok(Response::new(()))
    }

    async fn get_song_by_path(
        &self,
        request: Request<PathMessage>,
    ) -> Result<Response<SongResponse>, Status> {
        let manager = self.manager.read().await;
        let connection_type = get_connection_type(&request, &manager).await?;
        let request = request.into_inner();

        match &connection_type {
            ConnectionType::Local => match manager.get_song_by_path(url_decode(request.path)).await
            {
                Ok(Some(e)) => Ok(Response::new(SongResponse {
                    song: Some(map_lookup_entry(e, &connection_type)?),
                })),
                Ok(None) => Ok(Response::new(SongResponse { song: None })),
                Err(e) => Err(format_error(format!("Error getting track {e:?}"))),
            },
            ConnectionType::Remote {
                folders,
                local_addr,
            } => {
                let path = url_decode(request.path);
                for folder in folders {
                    match manager
                        .get_song_by_path(path.replace(local_addr, folder))
                        .await
                    {
                        Ok(Some(e)) => {
                            return Ok(Response::new(SongResponse {
                                song: Some(map_lookup_entry(e, &connection_type)?),
                            }));
                        }
                        Err(e) => return Err(format_error(format!("Error getting track {e:?}"))),
                        _ => {}
                    }
                }
                Ok(Response::new(SongResponse { song: None }))
            }
        }
    }
}

fn url_decode(url: String) -> String {
    if url.starts_with("https://") || url.starts_with("http://") {
        urlencoding::decode(&url).unwrap().to_string()
    } else {
        url
    }
}
