use crate::management_server::Management;

use crate::rpc::*;
use std::pin::Pin;

use futures::StreamExt;
use libplatune_management::config::Config;
use libplatune_management::database::Database;
use tonic::{Request, Response, Status};

pub struct ManagementImpl {
    config: Config,
}

impl ManagementImpl {
    pub async fn new() -> ManagementImpl {
        dotenv::from_path("./.env").unwrap_or_default();
        let path = std::env::var("DATABASE_URL").unwrap();
        let db = Database::connect(path).await;
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
}
