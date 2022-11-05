use crate::manager::Manager;
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use super::file_watch_error::FileWatchError;

#[derive(Clone)]
pub struct FileWatchManager {
    manager: Manager,
    watch_folder_tx: tokio::sync::mpsc::Sender<PathBuf>,
}

impl Deref for FileWatchManager {
    type Target = Manager;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

impl DerefMut for FileWatchManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.manager
    }
}

impl FileWatchManager {
    pub fn new(manager: Manager, watch_folder_tx: tokio::sync::mpsc::Sender<PathBuf>) -> Self {
        Self {
            manager,
            watch_folder_tx,
        }
    }
    pub async fn add_folder(&self, path: &str) -> Result<(), FileWatchError> {
        self.add_folders(vec![path]).await
    }

    pub async fn add_folders(&self, paths: Vec<&str>) -> Result<(), FileWatchError> {
        self.manager.add_folders(paths.clone()).await?;

        for path in paths {
            self.watch_folder_tx
                .send(PathBuf::from(path))
                .await
                .map_err(|e| FileWatchError::ThreadCommError(e.to_string()))?;
        }
        Ok(())
    }
}
