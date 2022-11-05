use crate::rpc::*;
use daemon_slayer::{
    server::BroadcastEventStore,
    task_queue::{JobProcessor, Xid},
};
use futures::StreamExt;
use libplatune_management::file_watcher::file_watch_manager::FileWatchManager;
use tap::TapFallible;
use tracing::error;

pub struct SyncProcessor {
    manager: FileWatchManager,
    progress_tx: tokio::sync::broadcast::Sender<Progress>,
}

impl SyncProcessor {
    pub fn new(manager: FileWatchManager) -> Self {
        let (progress_tx, _) = tokio::sync::broadcast::channel(32);
        Self {
            manager,
            progress_tx,
        }
    }

    pub fn get_event_store(&self) -> BroadcastEventStore<Progress> {
        BroadcastEventStore::new(self.progress_tx.clone())
    }
}

#[async_trait::async_trait]
impl JobProcessor for SyncProcessor {
    type Payload = Option<Vec<String>>;
    type Error = anyhow::Error;

    fn name() -> &'static str {
        "sync"
    }

    async fn handle(&self, _: Xid, paths: Self::Payload) -> Result<(), Self::Error> {
        let progress_tx = self.progress_tx.clone();
        let mut manager = self.manager.clone();
        manager
            .sync(paths, |mut progress_stream| async move {
                tokio::spawn(async move {
                    while let Some(m) = progress_stream.next().await {
                        progress_tx
                            .send(Progress {
                                job: "sync".to_string(),
                                percentage: m
                                    .tap_err(|e| error!("Error getting progress: {e:?}"))
                                    .unwrap_or(0.0),
                                finished: false,
                            })
                            //.tap_err(|e| warn!("unable to send {e:?}"))
                            .ok();
                    }
                    progress_tx
                        .send(Progress {
                            job: "sync".to_string(),
                            percentage: 1.0,
                            finished: true,
                        })
                        //.tap_err(|e| warn!("unable to send {e:?}"))
                        .ok();
                });
            })
            .await?;
        Ok(())
    }
}
