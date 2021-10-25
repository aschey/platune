use sqlx::{Pool, Sqlite};
use tokio::sync::mpsc::{channel, Receiver};

use super::sync_engine::SyncEngine;

#[derive(Clone)]
pub(crate) struct SyncController {
    pool: Pool<Sqlite>,
}

impl SyncController {
    pub(crate) fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
    pub(crate) async fn sync(&self, folders: Vec<String>, mount: Option<String>) -> Receiver<f32> {
        let (tx, rx) = channel(32);
        if !folders.is_empty() {
            let pool = self.pool.clone();

            tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                let mut engine = SyncEngine::new(folders, pool, mount, tx);
                rt.block_on(engine.start());
            });
        }

        rx
    }
}
