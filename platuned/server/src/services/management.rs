use crate::management_server::Management;
use crate::rpc::*;

use futures::StreamExt;
use libplatune_management::manager;
use libplatune_management::manager::{Manager, SearchOptions};
use notify::{DebouncedEvent, PollWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use prost_types::Timestamp;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc, RwLock};
use tonic::Request;
use tonic::Streaming;
use tonic::{Response, Status};
use tracing::{error, warn};

pub struct ManagementImpl {
    manager: ManagerWrapper,
    shutdown_tx: broadcast::Sender<()>,
    sync_tx: flume::Sender<SyncMessage>,
    progress_tx: broadcast::Sender<Progress>,
}

#[derive(Clone)]
pub(crate) struct ManagerWrapper {
    manager: Arc<RwLock<Manager>>,
    watcher: Arc<RecommendedWatcher>,
}

impl Deref for ManagerWrapper {
    type Target = RwLock<Manager>;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

impl ManagerWrapper {
    pub(crate) async fn new(
        manager: Manager,
        progress_tx: broadcast::Sender<Progress>,
        sync_tx: flume::Sender<SyncMessage>,
        sync_rx: flume::Receiver<SyncMessage>,
    ) -> Self {
        let (event_tx, event_rx) = std::sync::mpsc::channel();

        let mut watcher = RecommendedWatcher::new(event_tx, Duration::from_millis(5000)).unwrap();
        let paths = manager.get_all_folders().await.unwrap();
        for path in paths {
            watcher.watch(path, RecursiveMode::Recursive).unwrap();
        }

        thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                println!("{event:?}");
                match event {
                    DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => {
                        sync_tx.send(SyncMessage::Path(path)).unwrap();
                    }
                    _ => {}
                }
            }
        });
        let manager = Arc::new(RwLock::new(manager));
        let manager_ = manager.clone();

        tokio::spawn(async move {
            while let Ok(val) = sync_rx.recv_async().await {
                match val {
                    SyncMessage::All => {
                        let mut rx = manager_.write().await.sync(None).await.unwrap();
                        while let Some(m) = rx.next().await {
                            progress_tx
                                .send(Progress {
                                    job: "sync".to_string(),
                                    percentage: m.unwrap(),
                                })
                                .unwrap_or_default();
                        }
                    }
                    SyncMessage::Path(path) => {
                        let mut rx = manager_
                            .write()
                            .await
                            .sync(Some(vec![path.to_string_lossy().into_owned()]))
                            .await
                            .unwrap();
                        while let Some(m) = rx.next().await {
                            progress_tx
                                .send(Progress {
                                    job: "sync".to_string(),
                                    percentage: m.unwrap(),
                                })
                                .unwrap_or_default();
                        }
                    }
                }
            }
        });

        Self {
            manager,
            watcher: Arc::new(watcher),
        }
    }
}

impl ManagementImpl {
    pub(crate) fn new(
        manager: ManagerWrapper,
        shutdown_tx: broadcast::Sender<()>,
        sync_tx: flume::Sender<SyncMessage>,
        progress_tx: broadcast::Sender<Progress>,
    ) -> Self {
        Self {
            manager,
            shutdown_tx,
            sync_tx,
            progress_tx,
        }
    }
}

fn format_error(msg: String) -> Status {
    error!("{:?}", msg);
    Status::internal(msg)
}

pub(crate) enum SyncMessage {
    Path(PathBuf),
    All,
}

#[tonic::async_trait]
impl Management for ManagementImpl {
    async fn start_sync(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.sync_tx.send_async(SyncMessage::All).await.unwrap();
        Ok(Response::new(()))
    }

    type SubscribeEventsStream =
        Pin<Box<dyn futures::Stream<Item = Result<Progress, Status>> + Send + Sync + 'static>>;
    async fn subscribe_events(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
        let mut progress_rx = self.progress_tx.subscribe();
        let (tx, rx) = mpsc::channel(32);
        tokio::spawn(async move {
            loop {
                match progress_rx.recv().await {
                    Ok(val) => {
                        tx.send(Ok(val)).await.unwrap_or_default();
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

    // async fn sync(
    //     &self,
    //     request: Request<Streaming<()>>,
    // ) -> Result<Response<Self::SyncStream>, Status> {
    //     let mut messages = request.into_inner();
    //     let manager = self.manager.clone();
    //     // Close stream when shutdown is requested
    //     let mut shutdown_rx = self.shutdown_tx.subscribe();
    //     let mut file_changed_rx = self.file_changed_tx.subscribe();
    //     let (response_tx, response_rx) = tokio::sync::mpsc::channel(32);
    //     tokio::spawn(async move {
    //         while let Some(msg) = tokio::select! {
    //             val = messages.next() => match val { Some(Ok(_)) => Some(SyncMessage::All), _ => None },
    //             val = file_changed_rx.recv() => match val { Ok(val) => Some(SyncMessage::Path(val)), _ => None },
    //             _ = shutdown_rx.recv() => None
    //         } {
    //             let mut manager = manager.write().await;
    //             match msg {
    //                 SyncMessage::Path(path) => {
    //                     let mut rx = match manager.sync().await {
    //                         Ok(rx) => rx,
    //                         Err(e) => {
    //                             return Err(format_error(format!("Error syncing files {:?}", e)))
    //                         }
    //                     };
    //                     while let Some(r) = rx.next().await {
    //                         response_tx.send(r).await.unwrap();
    //                     }
    //                 }
    //                 SyncMessage::All => {
    //                     let mut rx = match manager.sync().await {
    //                         Ok(rx) => rx,
    //                         Err(e) => {
    //                             return Err(format_error(format!("Error syncing files {:?}", e)))
    //                         }
    //                     };
    //                     while let Some(r) = rx.next().await {
    //                         response_tx.send(r).await.unwrap();
    //                     }
    //                 }
    //             }
    //         }
    //         Ok(())
    //     });

    // Ok(Response::new(Box::pin(rx.map(
    //     |progress_result| match progress_result {
    //         Ok(percentage) => Ok(Progress { percentage }),
    //         Err(e) => Err(format_error(format!("Error syncing files {:?}", e))),
    //     },
    // ))))

    //     Ok(Response::new(Box::pin({
    //         tokio_stream::wrappers::ReceiverStream::new(response_rx).map(|progress_result| {
    //             match progress_result {
    //                 Ok(percentage) => Ok(Progress { percentage }),
    //                 Err(e) => Err(format_error(format!("Error syncing files {:?}", e))),
    //             }
    //         })
    //     })))
    // }

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
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        tokio::spawn(async move {
            while let Some(msg) =
                tokio::select! { val = messages.next() => val, _ = shutdown_rx.recv() => None }
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
