use crate::{
    sync_handler_builder::SyncHandlerBuilder, sync_handler_client::SyncHandlerClient,
    sync_processor::SyncProcessor,
};
use daemon_slayer::server::BackgroundService;

pub struct SyncHandler {
    tx: tokio::sync::mpsc::Sender<Option<Vec<String>>>,
    handle: tokio::task::JoinHandle<()>,
    shutdown_tx: tokio::sync::mpsc::Sender<()>,
}

#[async_trait::async_trait]
impl BackgroundService for SyncHandler {
    type Builder = SyncHandlerBuilder;
    type Client = SyncHandlerClient;

    async fn run_service(mut builder: Self::Builder) -> Self {
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel(32);
        let handle = tokio::spawn(async move {
            while let Some(paths) = tokio::select! { val = builder.path_rx.recv() => val, _ = shutdown_rx.recv() => None }
            {
                builder
                    .task_queue_client
                    .schedule::<SyncProcessor>(paths, 0)
                    .await;
            }
        });

        Self {
            tx: builder.path_tx,
            shutdown_tx,
            handle,
        }
    }

    fn get_client(&mut self) -> Self::Client {
        SyncHandlerClient::new(self.tx.clone())
    }

    async fn stop(self) {
        self.shutdown_tx.send(()).await.unwrap();
        self.handle.await.unwrap();
    }
}
