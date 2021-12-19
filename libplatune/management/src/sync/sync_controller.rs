use sqlx::{Pool, Sqlite};
use tokio::{
    runtime::Runtime,
    sync::{
        broadcast::{channel, Sender},
        oneshot,
    },
};
use tracing::error;

use super::{
    progress_stream::ProgressStream,
    sync_engine::{SyncEngine, SyncError},
};

pub(crate) struct SyncController {
    pool: Pool<Sqlite>,
    progress_tx: Option<Sender<Option<Result<f32, SyncError>>>>,
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
        if let Some(finished_rx) = &mut self.finished_rx {
            if finished_rx.try_recv().is_err() {
                if let Some(tx) = &self.progress_tx {
                    return ProgressStream::new(tx.subscribe());
                }
            }
        }
        let (finished_tx, finished_rx) = oneshot::channel();

        let (tx, rx) = channel(10000);
        self.finished_rx = Some(finished_rx);

        self.progress_tx = Some(tx.clone());
        if !folders.is_empty() {
            let pool = self.pool.clone();

            tokio::task::spawn_blocking(move || {
                let rt = Runtime::new().unwrap();
                let mut engine = SyncEngine::new(folders, pool, mount, tx);
                rt.block_on(engine.start());
                if let Err(e) = finished_tx.send(()) {
                    error!("Error sending sync finished signal {:?}", e);
                }
                finished_callback();
            });
        } else if let Err(e) = tx.send(None) {
            error!("Error sending sync finished signal {:?}", e);
        }

        ProgressStream::new(rx)
    }
}
