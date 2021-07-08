use crate::management_server::Management;
use crate::rpc::*;
use futures::StreamExt;
use libplatune_management::config::Config;
use libplatune_management::database::Database;
use std::pin::Pin;
use tokio::sync::mpsc;
use tonic::Request;
use tonic::Streaming;
use tonic::{Response, Status};

pub struct ManagementImpl {
    config: Config,
    db: Database,
}

impl ManagementImpl {
    pub async fn new() -> ManagementImpl {
        dotenv::from_path("./.env").unwrap_or_default();
        let path = std::env::var("DATABASE_URL").unwrap();
        let db = Database::connect(path, true).await;
        db.migrate().await;
        let config = Config::new(&db);
        ManagementImpl { config, db }
    }
}

#[tonic::async_trait]
impl Management for ManagementImpl {
    type SyncStream =
        Pin<Box<dyn futures::Stream<Item = Result<Progress, Status>> + Send + Sync + 'static>>;

    async fn sync(&self, _: Request<()>) -> Result<Response<Self::SyncStream>, Status> {
        let rx = self.config.sync().await;
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx).map(|r| Ok(Progress { percentage: r })),
        )))
    }

    async fn add_folders(&self, request: Request<FoldersMessage>) -> Result<Response<()>, Status> {
        self.config
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
        let folders = self.config.get_all_folders().await;
        Ok(Response::new(FoldersMessage { folders }))
    }

    async fn register_mount(
        &self,
        request: Request<RegisteredMountMessage>,
    ) -> Result<Response<()>, Status> {
        self.config
            .register_drive(&request.into_inner().mount)
            .await;
        Ok(Response::new(()))
    }

    async fn get_registered_mount(
        &self,
        _: Request<()>,
    ) -> Result<Response<RegisteredMountMessage>, Status> {
        let mount = self.config.get_registered_mount().await;
        Ok(Response::new(RegisteredMountMessage {
            mount: mount.unwrap_or_default(),
        }))
    }

    type SearchStream = Pin<
        Box<dyn futures::Stream<Item = Result<SearchResponse, Status>> + Send + Sync + 'static>,
    >;

    async fn search(
        &self,
        request: Request<Streaming<SearchRequest>>,
    ) -> Result<Response<Self::SearchStream>, Status> {
        let mut messages = request.into_inner();
        let db = self.db.clone();
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(msg) = messages.next().await {
                let msg = msg.unwrap();
                tx.send(db.search(&msg.query, Default::default()).await)
                    .await
                    .unwrap();
            }
        });

        Ok(Response::new(Box::pin({
            tokio_stream::wrappers::ReceiverStream::new(rx).map(|r| {
                let results = r
                    .into_iter()
                    .map(|res| SearchResult {
                        description: res.get_description(),
                        entry: res.get_formatted_entry(),
                        entry_type: (match &res.entry_type[..] {
                            "song" => EntryType::Song,
                            "artist" => EntryType::Artist,
                            "album_artist" => EntryType::AlbumArtist,
                            "album" => EntryType::Album,
                            _ => unreachable!("Unknown entry type"),
                        })
                        .into(),
                        artist: res.artist,
                        correlation_id: res.correlation_id,
                    })
                    .collect();
                Ok(SearchResponse { results })
            })
        })))
    }
}
