use crate::management_server::Management;
use crate::rpc::*;
use anyhow::{Context, Result};
use futures::StreamExt;
use libplatune_management::config::Config;
use libplatune_management::database::Database;
use libplatune_management::manager;
use libplatune_management::manager::{Manager, SearchOptions};
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::Request;
use tonic::Streaming;
use tonic::{Response, Status};
use tracing::{error, warn};

pub struct ManagementImpl {
    manager: Manager,
}

impl ManagementImpl {
    pub async fn try_new() -> Result<ManagementImpl> {
        let path = std::env::var("DATABASE_URL")
            .with_context(|| "DATABASE_URL environment variable not set")?;
        let db = Database::connect(path, true).await?;
        db.migrate()
            .await
            .with_context(|| "Error migrating database")?;
        let config = Config::try_new()?;
        let manager = Manager::new(&db, &config);
        Ok(ManagementImpl { manager })
    }
}

#[tonic::async_trait]
impl Management for ManagementImpl {
    type SyncStream =
        Pin<Box<dyn futures::Stream<Item = Result<Progress, Status>> + Send + Sync + 'static>>;

    async fn sync(&self, _: Request<()>) -> Result<Response<Self::SyncStream>, Status> {
        let rx = self.manager.sync().await;
        Ok(Response::new(Box::pin(ReceiverStream::new(rx).map(
            |progress_result| match progress_result {
                Ok(percentage) => Ok(Progress { percentage }),
                Err(e) => {
                    let msg = format!("Error syncing files {:?}", e);
                    error!("{}", msg);
                    Err(Status::internal(msg))
                }
            },
        ))))
    }

    async fn add_folders(&self, request: Request<FoldersMessage>) -> Result<Response<()>, Status> {
        self.manager
            .add_folders(
                request
                    .into_inner()
                    .folders
                    .iter()
                    .map(|s| &s[..])
                    .collect(),
            )
            .await;
        Ok(Response::new(()))
    }

    async fn get_all_folders(&self, _: Request<()>) -> Result<Response<FoldersMessage>, Status> {
        let folders = self.manager.get_all_folders().await;
        Ok(Response::new(FoldersMessage { folders }))
    }

    async fn register_mount(
        &self,
        request: Request<RegisteredMountMessage>,
    ) -> Result<Response<()>, Status> {
        match self
            .manager
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
        let mount = self.manager.get_registered_mount().await;
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
                let msg = format!("Error sending lookup request {:?}", e);
                error!("{}", &msg);
                return Err(Status::internal(msg));
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

        tokio::spawn(async move {
            while let Some(msg) = messages.next().await {
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
                        let msg = format!("Error sending search request {:?}", e);
                        return Err(Status::internal(msg));
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
}
