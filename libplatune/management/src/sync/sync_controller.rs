use super::{
    progress_stream::ProgressStream,
    sync_engine::{SyncEngine, SyncError},
};
use futures::Future;
use sqlx::{Pool, Sqlite};
use tap::TapFallible;
use tokio::sync::{broadcast, oneshot};
use tracing::{error, info, warn};

pub(crate) struct SyncController {
    write_pool: Pool<Sqlite>,
    progress_tx: Option<broadcast::Sender<Option<Result<f32, SyncError>>>>,
    finished_rx: Option<oneshot::Receiver<()>>,
}

impl SyncController {
    pub(crate) fn new(write_pool: Pool<Sqlite>) -> Self {
        Self {
            write_pool,
            progress_tx: None,
            finished_rx: None,
        }
    }
    pub(crate) async fn sync<F, Fut>(
        &mut self,
        folders: Vec<String>,
        mount: Option<String>,
        on_started: F,
    ) where
        F: FnOnce(ProgressStream) -> Fut,
        Fut: Future<Output = ()>,
    {
        // If sync is currently running, subscribe to the current stream instead of starting another one
        if let Some(finished_rx) = &mut self.finished_rx {
            // If the finished channel has a value, the last sync finished so we should restart
            // Otherwise, the sync is curently in progress
            if finished_rx.try_recv().is_err() {
                if let Some(tx) = &self.progress_tx {
                    info!("Subscribing to sync in progress");

                    on_started(ProgressStream::new(tx.subscribe())).await;
                    return;
                }
            }
        }
        let (finished_tx, finished_rx) = oneshot::channel();

        let (tx, rx) = broadcast::channel(10000);
        self.finished_rx = Some(finished_rx);

        self.progress_tx = Some(tx.clone());
        let stream = ProgressStream::new(rx);
        if !folders.is_empty() {
            let write_pool = self.write_pool.clone();

            info!("Starting new sync");
            let mut engine = SyncEngine::new(folders, write_pool, mount, tx.clone());
            on_started(stream).await;
            engine.start().await;

            let _ = finished_tx
                .send(())
                .tap_err(|e| info!("Couldn't send sync finished signal: {e:?}"));

            // Don't send final sync message until we've completed all of the cleanup steps
            let _ = tx
                .send(None)
                .tap_err(|e| warn!("Error sending final sync message to clients {e:?}"));
        } else {
            info!("No folders to sync");
            on_started(stream).await;
            let _ = tx
                .send(None)
                .tap_err(|e| error!("Error sending sync finished signal {e:?}"));
        }
    }
}
