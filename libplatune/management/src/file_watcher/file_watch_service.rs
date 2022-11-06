use crate::manager::Manager;
use daemon_slayer_core::server::{BackgroundService, SubsystemHandle};
use notify::{
    event::{EventKind, ModifyKind, RenameMode},
    RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{path::PathBuf, sync::Arc};
use tap::TapFallible;
use tracing::{error, info};

use super::{
    file_watch_builder::FileWatchBuilder, file_watch_error::FileWatchError,
    file_watch_manager::FileWatchManager,
};

#[derive(Debug)]
pub(crate) enum SyncMessage {
    Path(PathBuf),
    Rename(PathBuf, PathBuf),
}

#[derive(Clone, Debug)]
pub struct Progress {
    pub job: String,
    pub percentage: f32,
    pub finished: bool,
}

#[derive(Clone)]
pub struct FileWatchService {
    manager: Manager,
    watch_folder_tx: tokio::sync::mpsc::Sender<PathBuf>,
}

#[async_trait::async_trait]
impl BackgroundService for FileWatchService {
    type Builder = FileWatchBuilder;

    type Client = FileWatchManager;

    async fn run_service(builder: Self::Builder, subsys: SubsystemHandle) -> Self {
        let (event_tx, mut event_rx) = tokio::sync::mpsc::channel(32);
        let (sync_tx, mut sync_rx) = tokio::sync::mpsc::channel(32);
        let (watch_folder_tx, mut watch_folder_rx) = tokio::sync::mpsc::channel::<PathBuf>(32);
        let sync_tx_ = sync_tx.clone();

        let mut watcher = RecommendedWatcher::new(
            move |e| {
                event_tx
                    .blocking_send(e)
                    .tap_err(|e| error!("Error sending file watch event: {e:?}"))
                    .ok();
            },
            notify::Config::default(),
        )
        .map_err(FileWatchError::WatchError)
        .unwrap();

        let paths = builder
            .manager
            .get_all_folders()
            .await
            .map_err(FileWatchError::DbError)
            .unwrap();

        for path in &paths {
            watcher
                .watch(Path::new(path), RecursiveMode::Recursive)
                .tap_err(|e|   // Probably a bad file path
            error!("Error watching path {path}: {e:?}"))
                .ok();
        }

        tokio::spawn(async move {
            while let Some(path) = watch_folder_rx.recv().await {
                watcher
                    .watch(path.as_ref(), RecursiveMode::Recursive)
                    .tap_err(|e|   // Probably a bad file path
                        error!("Error watching path {path:?}: {e:?}"))
                    .ok();
            }
        });

        // TODO: add a way to terminate these child tasks
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                if let Ok(event) =
                    event.tap_err(|e| error!("Error received from file watcher: {e:?}"))
                {
                    info!("Received file watcher event {event:?}");
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

        let mut manager_ = builder.manager.clone();

        tokio::spawn(async move {
            let mut paths: Vec<PathBuf> = vec![];
            let running = Arc::new(AtomicBool::new(false));
            loop {
                let watch_event = tokio::time::timeout(builder.debounce_delay, sync_rx.recv())
                    .await
                    .tap_ok(|event| info!("Processing watch event: {event:?}"));

                match watch_event {
                    // Ok(Some(SyncMessage::All)) => {
                    //     path_tx.send(None).await;
                    // }
                    Ok(Some(SyncMessage::Path(new_path))) => {
                        paths = Self::normalize_paths(paths, new_path);
                    }
                    Ok(Some(SyncMessage::Rename(from, to))) => {
                        let _ = manager_
                            .rename_path(from, to.clone())
                            .await
                            .tap_err(|e| error!("Error renaming path: {e:?}"));

                        // Add new path to sync list in case the new path maps to paths that are currently marked as deleted
                        // So we need to now mark them as un-deleted
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
                            // Force sync all folders on mac because fsevents doesn't always track all events by design
                            None
                        } else {
                            let folders = paths
                                .iter()
                                .map(|p| p.to_string_lossy().into_owned())
                                .collect();
                            info!("Syncing {folders:?}");
                            Some(folders)
                        };

                        builder
                            .path_tx
                            .send(folders)
                            .await
                            .tap_err(|e| error!("Error sending folders: {e:?}"))
                            .ok();

                        paths.clear();
                    }
                }
            }
        });

        Self {
            watch_folder_tx,
            manager: builder.manager,
        }
    }

    fn get_client(&mut self) -> Self::Client {
        FileWatchManager::new(self.manager.clone(), self.watch_folder_tx.clone())
    }

    async fn stop(self) {}
}

impl FileWatchService {
    fn normalize_paths(paths: Vec<PathBuf>, new_path: PathBuf) -> Vec<PathBuf> {
        let mut new_paths = vec![];
        let mut add_new_path = true;
        // Only need to sync paths that are mutually exclusive
        // i.e. we don't need to sync /test/dir and /test/dir/1 separately because the second is a subset of the first
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
}

#[cfg(test)]
#[path = "./file_watch_manager_test.rs"]
mod file_watch_manager_test;
