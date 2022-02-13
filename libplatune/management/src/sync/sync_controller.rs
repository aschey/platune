use super::{
    progress_stream::ProgressStream,
    sync_engine::{SyncEngine, SyncError},
};
use sqlx::{Pool, Sqlite};
use tokio::sync::{broadcast, oneshot};
use tracing::{error, info};

pub(crate) struct SyncController {
    pool: Pool<Sqlite>,
    progress_tx: Option<broadcast::Sender<Option<Result<f32, SyncError>>>>,
    finished_rx: Option<oneshot::Receiver<()>>,
}

impl SyncController {
    pub(crate) fn new(pool: Pool<Sqlite>) -> Self {
        Self {
            pool,
            progress_tx: None,
            finished_rx: None,
        }
    }
    pub(crate) async fn sync(
        &mut self,
        folders: Vec<String>,
        mount: Option<String>,
        finished_callback: Box<dyn Fn() + Send>,
    ) -> ProgressStream {
        // If sync is currently running, subscribe to the current stream instead of starting another one
        if let Some(finished_rx) = &mut self.finished_rx {
            // If the finished channel has a value, the last sync finished so we should restart
            // Otherwise, the sync is curently in progress
            if finished_rx.try_recv().is_err() {
                if let Some(tx) = &self.progress_tx {
                    return ProgressStream::new(tx.subscribe());
                }
            }
        }
        let (finished_tx, finished_rx) = oneshot::channel();

        let (tx, rx) = broadcast::channel(10000);
        self.finished_rx = Some(finished_rx);

        self.progress_tx = Some(tx.clone());
        if !folders.is_empty() {
            let pool = self.pool.clone();

            tokio::task::spawn(async move {
                let mut engine = SyncEngine::new(folders, pool, mount, tx);
                engine.start().await;
                if finished_tx.send(()).is_err() {
                    info!("Couldn't send sync finished signal");
                }
                finished_callback();
            });
        } else if let Err(e) = tx.send(None) {
            error!("Error sending sync finished signal {e:?}");
        }

        ProgressStream::new(rx)
    }
}
