use crate::db_error::DbError;
use crate::manager::Manager;
use futures::StreamExt;
use notify::{DebouncedEvent, Watcher};
use notify::{RecommendedWatcher, RecursiveMode};
use std::ops::Deref;
use std::{path::PathBuf, sync::Arc, thread, time::Duration};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tracing::info;

#[derive(Debug)]
pub(crate) enum SyncMessage {
    Path(PathBuf),
    Rename(PathBuf, PathBuf),
    Hold,
    All,
}

#[derive(Clone, Debug)]
pub struct Progress {
    pub job: String,
    pub percentage: f32,
    pub finished: bool,
}

#[derive(Clone)]
pub struct FileWatchManager {
    manager: Arc<RwLock<Manager>>,
    watcher: Arc<Mutex<RecommendedWatcher>>,
    sync_tx: mpsc::Sender<SyncMessage>,
    progress_tx: broadcast::Sender<Progress>,
}

impl Deref for FileWatchManager {
    type Target = RwLock<Manager>;

    fn deref(&self) -> &Self::Target {
        &self.manager
    }
}

impl FileWatchManager {
    pub async fn new(manager: Manager, debounce_delay: Duration) -> Self {
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let (sync_tx, mut sync_rx) = tokio::sync::mpsc::channel(32);
        let sync_tx_ = sync_tx.clone();
        let (progress_tx, _) = broadcast::channel(32);
        let progress_tx_ = progress_tx.clone();

        let mut watcher = RecommendedWatcher::new(event_tx, debounce_delay).unwrap();
        let paths = manager.get_all_folders().await.unwrap();

        for path in &paths {
            watcher.watch(&path, RecursiveMode::Recursive).unwrap();
        }

        thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                info!("Received file watcher event {event:?}");
                match event {
                    DebouncedEvent::Create(path)
                    | DebouncedEvent::Write(path)
                    | DebouncedEvent::Remove(path) => {
                        sync_tx_.blocking_send(SyncMessage::Path(path)).unwrap();
                    }
                    DebouncedEvent::NoticeWrite(_) | DebouncedEvent::NoticeRemove(_) => {
                        // Write or remove pending, don't need to send the path yet but we can reset the debouncer
                        sync_tx_.blocking_send(SyncMessage::Hold).unwrap();
                    }
                    DebouncedEvent::Rename(from, to) => {
                        sync_tx_
                            .blocking_send(SyncMessage::Rename(from, to))
                            .unwrap();
                    }
                    _ => {}
                }
            }
        });
        let manager = Arc::new(RwLock::new(manager));
        let manager_ = manager.clone();

        tokio::spawn(async move {
            let mut paths: Vec<PathBuf> = vec![];
            loop {
                // Wait for longer than the debounce duration to ensure we get all the emitted events
                match tokio::time::timeout(debounce_delay * 2, sync_rx.recv()).await {
                    Ok(Some(SyncMessage::All)) => {
                        let mut rx = manager_.write().await.sync(None).await.unwrap();
                        while let Some(m) = rx.next().await {
                            progress_tx_
                                .send(Progress {
                                    job: "sync".to_string(),
                                    percentage: m.unwrap(),
                                    finished: false,
                                })
                                .unwrap_or_default();
                        }
                        progress_tx_
                            .send(Progress {
                                job: "sync".to_string(),
                                percentage: 1.0,
                                finished: true,
                            })
                            .unwrap_or_default();
                    }
                    Ok(Some(SyncMessage::Path(new_path))) => {
                        paths = Self::normalize_paths(paths, new_path);
                        info!("Paths to be synced: {paths:?}");
                    }
                    Ok(Some(SyncMessage::Rename(from, to))) => {
                        manager_
                            .write()
                            .await
                            .rename_path(from, to.clone())
                            .await
                            .unwrap();
                        // Add new path to sync list in case the new path maps to paths that are currently marked as deleted
                        // So we need to now mark them as un-deleted
                        paths = Self::normalize_paths(paths, to);
                    }
                    Ok(Some(SyncMessage::Hold)) => {}
                    Ok(None) => {
                        break;
                    }
                    Err(_) => {
                        if paths.is_empty() {
                            continue;
                        }

                        let folders = paths
                            .iter()
                            .map(|p| p.to_string_lossy().into_owned())
                            .collect();
                        let mut rx = manager_.write().await.sync(Some(folders)).await.unwrap();
                        while let Some(m) = rx.next().await {
                            progress_tx_
                                .send(Progress {
                                    job: "sync".to_string(),
                                    percentage: m.unwrap(),
                                    finished: false,
                                })
                                .unwrap_or_default();
                        }
                        progress_tx_
                            .send(Progress {
                                job: "sync".to_string(),
                                percentage: 1.0,
                                finished: true,
                            })
                            .unwrap_or_default();
                        paths.clear();
                    }
                }
            }
        });

        Self {
            manager,
            watcher: Arc::new(Mutex::new(watcher)),
            sync_tx,
            progress_tx,
        }
    }

    fn normalize_paths(paths: Vec<PathBuf>, new_path: PathBuf) -> Vec<PathBuf> {
        let mut new_paths = vec![];
        let mut add_new_path = true;
        // Only need to sync paths that are mutually exclusive
        // i.e. we don't need to sync /test/dir and /test/dir/1 separately because the second is a subset of the first
        for path in paths.into_iter() {
            // Keep the path if the new path is not an ancestor of this path
            if !path.starts_with(&new_path) {
                new_paths.push(path.clone());
            }
            // If a parent of this path is already being tracked, we don't need the new path
            if new_path.starts_with(path) {
                add_new_path = false;
            }
        }
        if add_new_path {
            new_paths.push(new_path);
        }
        info!("Paths to be synced: {new_paths:?}");
        new_paths
    }

    pub async fn start_sync_all(&self) {
        self.sync_tx.send(SyncMessage::All).await.unwrap();
    }

    pub fn subscribe_progress(&self) -> broadcast::Receiver<Progress> {
        self.progress_tx.subscribe()
    }

    pub async fn add_folder(&self, path: &str) -> Result<(), DbError> {
        self.add_folders(vec![path]).await
    }

    pub async fn add_folders(&self, paths: Vec<&str>) -> Result<(), DbError> {
        self.manager
            .write()
            .await
            .add_folders(paths.clone())
            .await?;

        for path in paths {
            self.watcher
                .lock()
                .await
                .watch(path, RecursiveMode::Recursive)
                .unwrap();
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "./file_watch_manager_test.rs"]
mod file_watch_manager_test;
