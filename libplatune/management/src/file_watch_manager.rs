use crate::manager::Manager;
use futures::StreamExt;
use notify::{DebouncedEvent, Watcher};
use notify::{RecommendedWatcher, RecursiveMode};
use std::ops::Deref;
use std::{path::PathBuf, sync::Arc, thread, time::Duration};
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::info;

#[derive(Debug)]
pub(crate) enum SyncMessage {
    Path(PathBuf),
    All,
}

#[derive(Clone)]
pub struct Progress {
    pub job: String,
    pub percentage: f32,
}

#[derive(Clone)]
pub struct FileWatchManager {
    manager: Arc<RwLock<Manager>>,
    watcher: Arc<RecommendedWatcher>,
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
    pub async fn new(manager: Manager) -> Self {
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let (sync_tx, mut sync_rx) = tokio::sync::mpsc::channel(32);
        let sync_tx_ = sync_tx.clone();
        let (progress_tx, _) = broadcast::channel(32);
        let progress_tx_ = progress_tx.clone();

        let mut watcher = RecommendedWatcher::new(event_tx, Duration::from_millis(2500)).unwrap();
        let paths = manager.get_all_folders().await.unwrap();
        for path in paths {
            watcher.watch(path, RecursiveMode::Recursive).unwrap();
        }

        thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                info!("Received file watcher event {event:?}");
                match event {
                    DebouncedEvent::Create(path)
                    | DebouncedEvent::Write(path)
                    | DebouncedEvent::Remove(path) => {
                        sync_tx_.try_send(SyncMessage::Path(path)).unwrap();
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
                match tokio::time::timeout(Duration::from_millis(5000), sync_rx.recv()).await {
                    Ok(Some(SyncMessage::All)) => {
                        let mut rx = manager_.write().await.sync(None).await.unwrap();
                        while let Some(m) = rx.next().await {
                            progress_tx_
                                .send(Progress {
                                    job: "sync".to_string(),
                                    percentage: m.unwrap(),
                                })
                                .unwrap_or_default();
                        }
                    }
                    Ok(Some(SyncMessage::Path(new_path))) => {
                        // New path is a parent of some existing path
                        // Replace existing path with new path
                        if let Some(index) = paths.iter().position(|p| p.starts_with(&new_path)) {
                            paths[index] = new_path;
                        }
                        // If parent of new path already exists, we don't need to add the new path
                        else if paths.iter().all(|p| !new_path.starts_with(p)) {
                            paths.push(new_path);
                        }
                        info!("Paths to be synced: {paths:?}");
                    }
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
                                })
                                .unwrap_or_default();
                        }
                        paths.clear();
                    }
                }
            }
        });

        Self {
            manager,
            watcher: Arc::new(watcher),
            sync_tx,
            progress_tx,
        }
    }

    pub async fn start_sync_all(&self) {
        self.sync_tx.send(SyncMessage::All).await.unwrap();
    }

    pub fn subscribe_progress(&self) -> broadcast::Receiver<Progress> {
        self.progress_tx.subscribe()
    }
}
