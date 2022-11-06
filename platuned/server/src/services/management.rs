use crate::management_server::Management;
use crate::rpc::*;
use crate::sync_handler_client::SyncHandlerClient;
use daemon_slayer::server::{BroadcastEventStore, EventStore, FutureExt, SubsystemHandle};
use daemon_slayer::signals::Signal;
use futures::StreamExt;
use libplatune_management::file_watcher::file_watch_manager::FileWatchManager;
use libplatune_management::manager::{self, SearchOptions};
use prost_types::Timestamp;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tonic::Request;
use tonic::Streaming;
use tonic::{Response, Status};
use tracing::{error, info, warn};

pub struct ManagementImpl {
    manager: Arc<RwLock<FileWatchManager>>,
    sync_client: SyncHandlerClient,
    progress_store: BroadcastEventStore<Progress>,
    subsys: SubsystemHandle,
}

impl ManagementImpl {
    pub(crate) fn new(
        manager: FileWatchManager,
        sync_client: SyncHandlerClient,
        progress_store: BroadcastEventStore<Progress>,
        subsys: SubsystemHandle,
    ) -> Self {
        Self {
            manager: Arc::new(RwLock::new(manager)),
            sync_client,
            progress_store,
            subsys,
        }
    }
}

fn format_error(msg: String) -> Status {
    error!("{:?}", msg);
    Status::internal(msg)
}

#[tonic::async_trait]
impl Management for ManagementImpl {
    async fn start_sync(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.sync_client.start_sync().await {
            Ok(_) => Ok(Response::new(())),
            Err(e) => Err(format_error(e.to_string())),
        }
    }

    type SubscribeEventsStream =
        Pin<Box<dyn futures::Stream<Item = Result<Progress, Status>> + Send + Sync + 'static>>;
    async fn subscribe_events(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
        let mut progress_rx = self.progress_store.subscribe_events();
        let subsys = self.subsys.clone();
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            info!("Starting event stream");
            while let Ok(Some(val)) = progress_rx.next().cancel_on_shutdown(&subsys).await {
                match val {
                    Ok(val) => tx.send(Ok(val)).await.unwrap_or_default(),
                    Err(BroadcastStreamRecvError::Lagged(_)) => continue,
                }
            }
            info!("Event stream completed");
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
            return Err(format_error(format!("Error adding folders {:?}", e)));
        };
        Ok(Response::new(()))
    }

    async fn get_all_folders(&self, _: Request<()>) -> Result<Response<FoldersMessage>, Status> {
        let folders = match self.manager.read().await.get_all_folders().await {
            Ok(f) => f,
            Err(e) => {
                return Err(format_error(format!("Error syncing files {:?}", e)));
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
            Err(e) => Err(Status::invalid_argument(format!("{}", e))),
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

    async fn lookup(
        &self,
        request: Request<LookupRequest>,
    ) -> Result<Response<LookupResponse>, Status> {
        let request = request.into_inner();
        let lookup_result = match self
            .manager
            .read()
            .await
            .lookup(
                request.correlation_ids,
                match EntryType::from_i32(request.entry_type).unwrap() {
                    EntryType::Song => manager::EntryType::Song,
                    EntryType::Album => manager::EntryType::Album,
                    EntryType::AlbumArtist => manager::EntryType::AlbumArtist,
                    EntryType::Artist => manager::EntryType::Artist,
                },
            )
            .await
        {
            Ok(entries) => entries,
            Err(e) => {
                return Err(format_error(format!(
                    "Error sending lookup request {:?}",
                    e
                )));
            }
        };
        let entries = lookup_result
            .into_iter()
            .map(|e| LookupEntry {
                artist: e.artist,
                album_artist: e.album_artist,
                album: e.album,
                song: e.song,
                path: e.path,
                track: e.track,
                duration: Some(Timestamp {
                    seconds: Duration::from_millis(e.duration_millis as u64).as_secs() as i64,
                    nanos: Duration::from_millis(e.duration_millis as u64).subsec_nanos() as i32,
                }),
            })
            .collect();
        Ok(Response::new(LookupResponse { entries }))
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
        let subsys = self.subsys.clone();

        tokio::spawn(async move {
            while let Ok(Some(msg)) = messages.next().cancel_on_shutdown(&subsys).await {
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
                        return Err(format_error(format!(
                            "Error sending search request {:?}",
                            e
                        )));
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
                            manager::EntryType::AlbumArtist => EntryType::AlbumArtist,
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
            Err(e) => return Err(format_error(format!("Error getting deleted songs {:?}", e))),
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
            return Err(format_error(format!("Error deleting tracks {:?}", e)));
        }

        Ok(Response::new(()))
    }

    async fn get_song_by_path(
        &self,
        request: Request<PathMessage>,
    ) -> Result<Response<SongResponse>, Status> {
        let request = request.into_inner();

        let manager = self.manager.read().await;
        match manager.get_song_by_path(request.path).await {
            Ok(Some(e)) => Ok(Response::new(SongResponse {
                song: Some(LookupEntry {
                    artist: e.artist,
                    album_artist: e.album_artist,
                    album: e.album,
                    song: e.song,
                    path: e.path,
                    track: e.track,
                    duration: Some(Timestamp {
                        seconds: Duration::from_millis(e.duration_millis as u64).as_secs() as i64,
                        nanos: Duration::from_millis(e.duration_millis as u64).subsec_nanos()
                            as i32,
                    }),
                }),
            })),
            Ok(None) => Ok(Response::new(SongResponse { song: None })),
            Err(e) => Err(format_error(format!("Error getting track {:?}", e))),
        }
    }
}
