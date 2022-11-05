use std::time::Duration;

use crate::manager::Manager;

pub struct FileWatchBuilder {
    pub(super) manager: Manager,
    pub(super) debounce_delay: Duration,
    pub(super) path_tx: tokio::sync::mpsc::Sender<Option<Vec<String>>>,
}

impl FileWatchBuilder {
    pub fn new(manager: Manager, path_tx: tokio::sync::mpsc::Sender<Option<Vec<String>>>) -> Self {
        Self {
            manager,
            path_tx,
            debounce_delay: Duration::from_millis(500),
        }
    }
}
