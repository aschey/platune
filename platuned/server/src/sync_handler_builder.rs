use daemon_slayer::task_queue::TaskQueueClient;
use libplatune_management::file_watcher::file_watch_manager::FileWatchManager;

pub struct SyncHandlerBuilder {
    pub manager: FileWatchManager,
    pub task_queue_client: TaskQueueClient,
    pub path_tx: tokio::sync::mpsc::Sender<Option<Vec<String>>>,
    pub path_rx: tokio::sync::mpsc::Receiver<Option<Vec<String>>>,
}

impl SyncHandlerBuilder {
    pub fn new(
        manager: FileWatchManager,
        task_queue_client: TaskQueueClient,
        path_tx: tokio::sync::mpsc::Sender<Option<Vec<String>>>,
        path_rx: tokio::sync::mpsc::Receiver<Option<Vec<String>>>,
    ) -> Self {
        Self {
            manager,
            task_queue_client,
            path_tx,
            path_rx,
        }
    }
}
