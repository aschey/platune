use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use futures::StreamExt;
use notify::event::{EventKind, ModifyKind, RenameMode};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tap::TapFallible;
use thiserror::Error;
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};
use tracing::{error, info, warn};

use crate::db_error::DbError;
use crate::manager::{Manager, ManagerError};
use crate::sync::progress_stream::ProgressStream;

#[derive(Debug)]
pub(crate) enum SyncMessage {
    Path(PathBuf),
    Rename(PathBuf, PathBuf),
    All,
}

#[derive(Clone, Debug)]
pub struct Progress {
    pub job: String,
    pub percentage: f32,
    pub finished: bool,
}

#[derive(Error, Debug)]
pub enum FileWatchError {
    #[error(transparent)]
    WatchError(#[from] notify::Error),
    #[error(transparent)]
    DbError(#[from] DbError),
    #[error(transparent)]
    ManagerError(#[from] ManagerError),
    #[error("Thread communication error: {0}")]
    ThreadCommError(String),
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
    pub async fn new<F>(
        manager: Manager,
        debounce_delay: Duration,
        finished_callback: F,
    ) -> Result<Self, FileWatchError>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + 'static,
    {
        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(32);
        let (sync_tx, mut sync_rx) = tokio::sync::mpsc::channel(32);
        let sync_tx_ = sync_tx.clone();
        let (progress_tx, _) = broadcast::channel(32);
        let progress_tx_ = progress_tx.clone();

        let mut watcher = RecommendedWatcher::new(
            move |event| {
                event_tx
                    .blocking_send(event)
                    .tap_err(|e| error!("Failed to send watcher event: {e:?}"))
                    .ok();
            },
            notify::Config::default(),
        )
        .map_err(FileWatchError::WatchError)?;

        let paths = manager
            .get_all_folders()
            .await
            .map_err(FileWatchError::DbError)?;

        for path in &paths {
            if let Err(e) = watcher.watch(Path::new(path), RecursiveMode::Recursive) {
                // Probably a bad file path
                error!("Error watching path {path}: {e:?}");
            }
        }

        // TODO: add a way to terminate these child tasks
        tokio::spawn(async move {
            let mut from_path: Option<PathBuf> = None;
            while let Some(event) = event_rx.recv().await {
                if let Ok(event) =
                    event.tap_err(|e| error!("Error received from file watcher: {e:?}"))
                {
                    if !matches!(event.kind, EventKind::Access(_)) {
                        info!("Received file watcher event {event:?}");
                    }
                    match event.kind {
                        EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => {
                            let _ = sync_tx_
                                .send(SyncMessage::Rename(
                                    event.paths[0].clone(),
                                    event.paths[1].clone(),
                                ))
                                .await
                                .tap_err(|e| error!("Error sending rename message: {e:?}"));
                        }
                        EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                            info!("Got rename::from event, waiting for rename::to");
                            // Store the from path so we can match it up with the to path
                            from_path = Some(event.paths[0].clone());
                        }
                        EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                            if let Some(from_path) = from_path.take() {
                                let _ = sync_tx_
                                    .send(SyncMessage::Rename(from_path, event.paths[0].clone()))
                                    .await
                                    .tap_err(|e| error!("Error sending rename message: {e:?}"));
                            } else {
                                warn!("Got a rename::to message without a rename::from");
                                for path in event.paths {
                                    let _ = sync_tx_
                                        .send(SyncMessage::Path(path))
                                        .await
                                        .tap_err(|e| error!("Error sending path message: {e:?}"));
                                }
                            }
                        }
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                            for path in event.paths {
                                let _ = sync_tx_
                                    .send(SyncMessage::Path(path))
                                    .await
                                    .tap_err(|e| error!("Error sending path message: {e:?}"));
                            }
                        }
                        _ => {}
                    }
                }
            }
        });
        let manager = Arc::new(RwLock::new(manager));
        let manager_ = manager.clone();

        tokio::spawn(async move {
            let mut paths: Vec<PathBuf> = vec![];
            let running = Arc::new(AtomicBool::new(false));
            loop {
                let watch_event = tokio::time::timeout(debounce_delay, sync_rx.recv())
                    .await
                    .tap_ok(|event| info!("Processing watch event: {event:?}"));

                match watch_event {
                    Ok(Some(SyncMessage::All)) => {
                        if let Ok(rx) = manager_
                            .write()
                            .await
                            .sync(None, finished_callback())
                            .await
                            .tap_err(|e| error!("Error syncing: {e:?}"))
                        {
                            Self::send_progress(running.clone(), progress_tx_.clone(), rx);
                        }
                    }
                    Ok(Some(SyncMessage::Path(new_path))) => {
                        paths = Self::normalize_paths(paths, new_path);
                    }
                    Ok(Some(SyncMessage::Rename(from, to))) => {
                        let _ = manager_
                            .write()
                            .await
                            .rename_path(from, to.clone())
                            .await
                            .tap_err(|e| error!("Error renaming path: {e:?}"));

                        // Add new path to sync list in case the new path maps to paths that are
                        // currently marked as deleted So we need to now
                        // mark them as un-deleted
                        paths = Self::normalize_paths(paths, to);
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(_) => {
                        if paths.is_empty() {
                            // If no paths were changed, no need to sync
                            continue;
                        }
                        if running.load(Ordering::SeqCst) {
                            info!("Sync already running, will start sync on next debounce timeout");
                            continue;
                        }

                        let folders = if cfg!(target_os = "macos") {
                            info!("Syncing all folders");
                            // Force sync all folders on mac because FSEvents doesn't always track
                            // all events by design
                            None
                        } else {
                            let folders = paths
                                .iter()
                                .map(|p| p.to_string_lossy().into_owned())
                                .collect();
                            info!("Syncing {folders:?}");
                            Some(folders)
                        };

                        if let Ok(rx) = manager_
                            .write()
                            .await
                            .sync(folders, finished_callback())
                            .await
                            .tap_err(|e| error!("Error syncing: {e:?}"))
                        {
                            Self::send_progress(running.clone(), progress_tx_.clone(), rx);
                        }

                        paths.clear();
                    }
                }
            }
        });

        Ok(Self {
            manager,
            watcher: Arc::new(Mutex::new(watcher)),
            sync_tx,
            progress_tx,
        })
    }

    fn send_progress(
        running: Arc<AtomicBool>,
        progress_tx: broadcast::Sender<Progress>,
        mut rx: ProgressStream,
    ) {
        tokio::spawn(async move {
            running.store(true, Ordering::SeqCst);
            while let Some(m) = rx.next().await {
                progress_tx
                    .send(Progress {
                        job: "sync".to_string(),
                        percentage: m
                            .tap_err(|e| error!("Error getting progress: {e:?}"))
                            .unwrap_or(0.0),
                        finished: false,
                    })
                    .unwrap_or_default();
            }
            progress_tx
                .send(Progress {
                    job: "sync".to_string(),
                    percentage: 1.0,
                    finished: true,
                })
                .unwrap_or_default();
            running.store(false, Ordering::SeqCst);
        });
    }

    fn normalize_paths(paths: Vec<PathBuf>, new_path: PathBuf) -> Vec<PathBuf> {
        let mut new_paths = vec![];
        let mut add_new_path = true;
        // Only need to sync paths that are mutually exclusive
        // i.e. we don't need to sync /test/dir and /test/dir/1 separately because the second is a
        // subset of the first
        for path in paths.into_iter() {
            // Keep the path if the new path is not an ancestor of this path
            if !path.starts_with(&new_path) || path == new_path {
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

        new_paths
    }

    pub async fn start_sync_all(&self) -> Result<(), FileWatchError> {
        self.sync_tx
            .send(SyncMessage::All)
            .await
            .map_err(|e| FileWatchError::ThreadCommError(e.to_string()))
    }

    pub fn subscribe_progress(&self) -> broadcast::Receiver<Progress> {
        self.progress_tx.subscribe()
    }

    pub async fn add_folder(&self, path: &str) -> Result<(), FileWatchError> {
        self.add_folders(vec![path]).await
    }

    pub async fn add_folders(&self, paths: Vec<&str>) -> Result<(), FileWatchError> {
        self.manager
            .write()
            .await
            .add_folders(paths.clone())
            .await?;

        let mut watcher = self.watcher.lock().await;
        for path in paths {
            watcher
                .watch(Path::new(path), RecursiveMode::Recursive)
                .map_err(FileWatchError::WatchError)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "./file_watch_manager_test.rs"]
mod file_watch_manager_test;
