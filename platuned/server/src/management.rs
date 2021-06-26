use crate::management_server::Management;

use crate::rpc::*;
use std::pin::Pin;

use futures::StreamExt;
use libplatune_management::config::Config;
use libplatune_management::database::Database;
use tonic::{Response, Status};

pub struct ManagementImpl {
    config: Config,
}

impl ManagementImpl {
    pub async fn new() -> ManagementImpl {
        dotenv::from_path("./.env").unwrap_or_default();
        let path = std::env::var("DATABASE_URL").unwrap();
        let db = Database::connect(path, true).await;
        db.migrate().await;
        let config = Config::new(&db);
        ManagementImpl { config }
    }
}

#[tonic::async_trait]
impl Management for ManagementImpl {
    type SyncStream =
        Pin<Box<dyn futures::Stream<Item = Result<Progress, Status>> + Send + Sync + 'static>>;

    async fn sync(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<Self::SyncStream>, tonic::Status> {
        let rx = self.config.sync().await;
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx).map(|r| Ok(Progress { percentage: r })),
        )))
    }

    async fn add_folders(
        &self,
        request: tonic::Request<crate::rpc::FoldersMessage>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
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

    async fn get_all_folders(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<crate::rpc::FoldersMessage>, tonic::Status> {
        let folders = self.config.get_all_folders().await;
        Ok(Response::new(FoldersMessage { folders }))
    }

    async fn register_mount(
        &self,
        request: tonic::Request<crate::rpc::RegisteredMountMessage>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        self.config
            .register_drive(&request.into_inner().mount)
            .await;
        Ok(Response::new(()))
    }

    async fn get_registered_mount(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<crate::rpc::RegisteredMountMessage>, tonic::Status> {
        let drive_id = self.config.get_drive_id();
        Ok(Response::new(RegisteredMountMessage {
            mount: drive_id.unwrap_or_default(),
        }))
    }
}
