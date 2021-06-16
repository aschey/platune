use crate::management_server::Management;

use crate::rpc::*;
use std::pin::Pin;

use futures::StreamExt;
use tonic::{Request, Response, Status};

pub struct ManagementImpl {}

impl ManagementImpl {
    pub fn new() -> ManagementImpl {
        ManagementImpl {}
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
        let rx = libplatune_management::sync();
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx)
                .map(|r| Ok(Progress { percentage: 1. })),
        )))
    }
}
