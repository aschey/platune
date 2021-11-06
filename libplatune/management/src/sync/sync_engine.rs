use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use itertools::Itertools;
use katatsuki::Track;
use regex::Regex;
use sqlx::{Pool, Sqlite};
use thiserror::Error;
use tokio::{
    sync::{
        broadcast,
        mpsc::{channel, Receiver, Sender},
    },
    task::JoinHandle,
    time::timeout,
};
use tracing::{error, info};

use crate::{consts::MIN_WORDS, db_error::DbError};

use super::{dir_read::DirRead, sync_dal::SyncDAL};

#[derive(Error, Debug, Clone)]
pub enum SyncError {
    #[error(transparent)]
    DbError(#[from] DbError),
    #[error("Async error: {0}")]
    AsyncError(String),
    #[error("IO Error: {0}")]
    IOError(String),
}

trait SendSyncError {
    fn send_error(&self, err: SyncError);
}

impl SendSyncError for broadcast::Sender<Option<Result<f32, SyncError>>> {
    fn send_error(&self, err: SyncError) {
        error!("{:?}", err);
        if let Err(e) = self.send(Some(Err(err))) {
            error!("Error sending broadcast message to clients {:?}", e);
        }
    }
}

pub(crate) struct SyncEngine {
    paths: Vec<String>,
    pool: Pool<Sqlite>,
    mount: Option<String>,
    tx: broadcast::Sender<Option<Result<f32, SyncError>>>,
    dispatch_tx: async_channel::Sender<Option<PathBuf>>,
    dispatch_rx: async_channel::Receiver<Option<PathBuf>>,
    finished_tx: Sender<DirRead>,
    finished_rx: Receiver<DirRead>,
}

impl SyncEngine {
    pub(crate) fn new(
        paths: Vec<String>,
        pool: Pool<Sqlite>,
        mount: Option<String>,
        tx: broadcast::Sender<Option<Result<f32, SyncError>>>,
    ) -> Self {
        let (dispatch_tx, dispatch_rx) = async_channel::bounded(100);
        let (finished_tx, finished_rx) = channel(100);

        Self {
            paths,
            pool,
            mount,
            tx,
            dispatch_tx,
            dispatch_rx,
            finished_tx,
            finished_rx,
        }
    }

    pub(crate) async fn start(&mut self) {
        let (tags_tx, tags_rx) = channel(100);
        let tags_handle = self.tags_task(tags_rx);

        for path in &self.paths {
            if let Err(e) = self.dispatch_tx.send(Some(PathBuf::from(path))).await {
                self.tx.send_error(SyncError::AsyncError(e.to_string()));
            }
        }
        if let Err(e) = self.task_loop(&tags_tx).await {
            self.tx.send_error(e);
        }

        if let Err(e) = tags_tx.send(None).await {
            self.tx.send_error(SyncError::AsyncError(e.to_string()));
        }

        match tags_handle.await {
            Ok(Err(e)) => self.tx.send_error(SyncError::DbError(e)),
            Ok(Ok(())) => {}
            Err(e) => {
                self.tx.send_error(SyncError::AsyncError(e.to_string()));
            }
        }
        if let Err(e) = self.tx.send(None) {
            error!("Error sending message to clients {:?}", e);
        }
    }

    async fn task_loop(
        &mut self,
        tags_tx: &Sender<Option<(Track, String)>>,
    ) -> Result<(), SyncError> {
        let mut num_tasks = 1;
        let max_tasks = 100;

        let mut handles = vec![];
        for _ in 0..num_tasks {
            handles.push(self.spawn_task(tags_tx.clone()));
        }

        let mut total_dirs = 0;
        let mut dirs_processed = 0;
        loop {
            match timeout(Duration::from_millis(1), self.finished_rx.recv()).await {
                Ok(Some(DirRead::Completed)) => {
                    dirs_processed += 1;

                    // edge case - entire dir is empty
                    if total_dirs == 0 {
                        self.tx
                            .send(Some(Ok(1.)))
                            .map_err(|e| SyncError::AsyncError(e.to_string()))?;
                        break;
                    }
                    self.tx
                        .send(Some(Ok((dirs_processed as f32) / (total_dirs as f32))))
                        .map_err(|e| SyncError::AsyncError(e.to_string()))?;

                    if total_dirs == dirs_processed {
                        break;
                    }
                }
                Ok(Some(DirRead::Found)) => {
                    total_dirs += 1;
                    self.tx
                        .send(Some(Ok((dirs_processed as f32) / (total_dirs as f32))))
                        .map_err(|e| SyncError::AsyncError(e.to_string()))?;
                }
                Ok(None) => {
                    break;
                }
                Err(_) => {
                    if num_tasks < max_tasks {
                        handles.push(self.spawn_task(tags_tx.clone()));
                        num_tasks += 1;
                        info!("Spawning task. Num tasks: {:?}", num_tasks);
                    }
                }
            }
        }

        for _ in 0..handles.len() {
            self.dispatch_tx
                .send(None)
                .await
                .map_err(|e| SyncError::AsyncError(e.to_string()))?;
        }
        for handle in handles {
            if let Err(e) = handle
                .await
                .map_err(|e| SyncError::AsyncError(e.to_string()))?
            {
                return Err(e);
            }
        }

        Ok(())
    }

    fn tags_task(
        &self,
        mut tags_rx: Receiver<Option<(Track, String)>>,
    ) -> JoinHandle<Result<(), DbError>> {
        let pool = self.pool.clone();

        tokio::spawn(async move {
            let mut dal = SyncDAL::try_new(pool).await?;
            while let Some(metadata) = tags_rx.recv().await {
                match metadata {
                    Some((metadata, path)) => {
                        let fingerprint =
                            metadata.artist.clone() + &metadata.album + &metadata.title;
                        dal.add_artist(&metadata.artist).await?;

                        dal.add_album_artist(&metadata.album_artists).await?;

                        dal.add_album(&metadata.album, &metadata.album_artists)
                            .await?;
                        dal.sync_song(&path, &metadata, &fingerprint).await?;
                    }
                    None => {
                        break;
                    }
                }
            }

            // TODO: delete missing songs
            dal.get_missing_songs().await?;

            dal.sync_spellfix().await?;
            SyncEngine::add_search_aliases(&mut dal).await?;

            info!("committing");
            dal.commit().await?;
            info!("done");

            Ok(())
        })
    }

    async fn add_search_aliases(dal: &mut SyncDAL<'_>) -> Result<(), DbError> {
        let long_vals = dal.get_long_entries().await?;

        let re = Regex::new(r"[\s-]+").unwrap();
        for entry_value in long_vals {
            let words = re.split(&entry_value).collect_vec();
            if words.len() < MIN_WORDS {
                continue;
            }
            let acronym = words
                .into_iter()
                .map(|w| {
                    if w == "and" {
                        "&".to_owned()
                    } else {
                        w.chars()
                            .next()
                            .unwrap_or_default()
                            .to_string()
                            .to_lowercase()
                    }
                })
                .collect_vec()
                .join("");

            dal.insert_alias(&entry_value, &acronym).await?;
        }

        Ok(())
    }

    fn spawn_task(
        &self,
        tags_tx: Sender<Option<(Track, String)>>,
    ) -> JoinHandle<Result<(), SyncError>> {
        let mount = self.mount.clone();
        let dispatch_tx = self.dispatch_tx.clone();
        let dispatch_rx = self.dispatch_rx.clone();
        let finished_tx = self.finished_tx.clone();
        tokio::spawn(async move {
            while let Ok(path) = dispatch_rx.recv().await {
                match path {
                    Some(path) => {
                        SyncEngine::parse_dir(path, &mount, &tags_tx, &dispatch_tx, &finished_tx)
                            .await?;
                    }
                    None => {
                        break;
                    }
                }
            }

            Ok(())
        })
    }

    async fn parse_dir(
        path: PathBuf,
        mount: &Option<String>,
        tags_tx: &Sender<Option<(Track, String)>>,
        dispatch_tx: &async_channel::Sender<Option<PathBuf>>,
        finished_tx: &Sender<DirRead>,
    ) -> Result<(), SyncError> {
        for dir_result in path
            .read_dir()
            .map_err(|e| SyncError::IOError(e.to_string()))?
        {
            let dir = dir_result.map_err(|e| SyncError::IOError(e.to_string()))?;

            if dir
                .file_type()
                .map_err(|e| SyncError::IOError(e.to_string()))?
                .is_file()
            {
                let file_path = dir.path();

                if let Some(metadata) = SyncEngine::parse_metadata(&file_path)? {
                    let file_path_str = SyncEngine::clean_file_path(&file_path, mount);
                    tags_tx
                        .send(Some((metadata, file_path_str)))
                        .await
                        .map_err(|e| SyncError::AsyncError(e.to_string()))?;
                }
            } else {
                dispatch_tx
                    .send(Some(dir.path()))
                    .await
                    .map_err(|e| SyncError::AsyncError(e.to_string()))?;
                finished_tx
                    .send(DirRead::Found)
                    .await
                    .map_err(|e| SyncError::AsyncError(e.to_string()))?;
            }
        }

        Ok(finished_tx
            .send(DirRead::Completed)
            .await
            .map_err(|e| SyncError::AsyncError(e.to_string()))?)
    }

    fn parse_metadata(file_path: &Path) -> Result<Option<Track>, SyncError> {
        let name = file_path.extension().unwrap_or_default();
        let _size = file_path
            .metadata()
            .map_err(|e| SyncError::IOError(e.to_string()))?
            .len();
        let mut song_metadata: Option<Track> = None;
        match &name.to_str().unwrap_or_default().to_lowercase()[..] {
            "mp3" | "m4a" | "ogg" | "wav" | "flac" | "aac" => {
                let tag_result = Track::from_path(file_path, None);
                match tag_result {
                    Err(e) => {
                        error!("{:?}", e);
                    }
                    Ok(tag) => {
                        song_metadata = Some(tag);
                    }
                }
            }

            _ => {}
        }

        Ok(song_metadata)
    }

    fn clean_file_path(file_path: &Path, mount: &Option<String>) -> String {
        let mut file_path_str = file_path.to_string_lossy().to_string();
        if cfg!(windows) {
            file_path_str = file_path_str.replace(r"\", r"/");
        }

        if let Some(ref mount) = mount {
            if file_path_str.starts_with(&mount[..]) {
                file_path_str = file_path_str.replace(&mount[..], "");
            }
        }

        file_path_str
    }
}
