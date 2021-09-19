use crate::management_server::Management;
use crate::rpc::*;
use futures::StreamExt;
use libplatune_management::config::Config;
use libplatune_management::database;
use libplatune_management::database::Database;
use libplatune_management::database::SearchOptions;
use libplatune_management::manager::Manager;
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::Request;
use tonic::Streaming;
use tonic::{Response, Status};

pub struct ManagementImpl {
    manager: Manager,
}

impl ManagementImpl {
    pub async fn new(env_path: &str) -> ManagementImpl {
        dotenv::from_path(env_path).unwrap_or_default();
        let path = std::env::var("DATABASE_URL").unwrap();
        let db = Database::connect(path, true).await;
        db.migrate().await;
        let config = Config::new();
        let manager = Manager::new(&db, &config);
        ManagementImpl { manager }
    }
}

#[tonic::async_trait]
impl Management for ManagementImpl {
    type SyncStream =
        Pin<Box<dyn futures::Stream<Item = Result<Progress, Status>> + Send + Sync + 'static>>;

    async fn sync(&self, _: Request<()>) -> Result<Response<Self::SyncStream>, Status> {
        let rx = self.manager.sync().await;
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx).map(|r| Ok(Progress { percentage: r })),
        )))
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
        let entries = self
            .manager
            .lookup(
                request.correlation_ids,
                match EntryType::from_i32(request.entry_type).unwrap() {
                    EntryType::Song => database::EntryType::Song,
                    EntryType::Album => database::EntryType::Album,
                    EntryType::AlbumArtist => database::EntryType::AlbumArtist,
                    EntryType::Artist => database::EntryType::Artist,
                },
            )
            .await
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
                            title_max_length: msg.title_max_length.map(|l| l as usize),
                            description_max_length: msg.description_max_length.map(|l| l as usize),
                            ..Default::default()
                        };
                        tx.send(manager.search(&msg.query, options).await)
                            .await
                            .unwrap();
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Response::new(Box::pin({
            tokio_stream::wrappers::ReceiverStream::new(rx).map(|r| {
                let results = r
                    .into_iter()
                    .map(|res| SearchResult {
                        description: res.description,
                        entry: res.entry,
                        entry_type: (match res.entry_type {
                            database::EntryType::Song => EntryType::Song,
                            database::EntryType::Artist => EntryType::Artist,
                            database::EntryType::AlbumArtist => EntryType::AlbumArtist,
                            database::EntryType::Album => EntryType::Album,
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
