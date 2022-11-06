use crate::{
    sync_handler_builder::SyncHandlerBuilder, sync_handler_client::SyncHandlerClient,
    sync_processor::SyncProcessor,
};
use daemon_slayer::server::{BackgroundService, FutureExt, SubsystemHandle};

pub struct SyncHandler {
    tx: tokio::sync::mpsc::Sender<Option<Vec<String>>>,
    handle: tokio::task::JoinHandle<()>,
}

#[async_trait::async_trait]
impl BackgroundService for SyncHandler {
    type Builder = SyncHandlerBuilder;
    type Client = SyncHandlerClient;

    async fn run_service(mut builder: Self::Builder, subsys: SubsystemHandle) -> Self {
        let handle = tokio::spawn(async move {
            while let Ok(Some(paths)) = builder.path_rx.recv().cancel_on_shutdown(&subsys).await {
                builder
                    .task_queue_client
                    .schedule::<SyncProcessor>(paths, 0)
                    .await;
            }
        });

        Self {
            tx: builder.path_tx,
            handle,
        }
    }

    fn get_client(&mut self) -> Self::Client {
        SyncHandlerClient::new(self.tx.clone())
    }

    async fn stop(self) {
        self.handle.await.unwrap();
    }
}
