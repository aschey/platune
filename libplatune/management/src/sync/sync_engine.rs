use ignore::WalkBuilder;
use itertools::Itertools;
use katatsuki::ReadOnlyTrack;
use regex::Regex;
use sqlx::{Pool, Sqlite};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::{sync::broadcast, task::JoinHandle};
use tracing::{error, info, warn};
use walkdir::WalkDir;

use crate::{consts::MIN_WORDS, db_error::DbError, path_util::clean_file_path};

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
}

impl SyncEngine {
    pub(crate) fn new(
        paths: Vec<String>,
        pool: Pool<Sqlite>,
        mount: Option<String>,
        tx: broadcast::Sender<Option<Result<f32, SyncError>>>,
    ) -> Self {
        Self {
            paths,
            pool,
            mount,
            tx,
        }
    }

    pub(crate) async fn start(&mut self) {
        info!("Starting sync process");

        if self.paths.is_empty() {
            return;
        }

        let mut walker_builder = WalkBuilder::new(&self.paths[0]);
        walker_builder.threads(10).standard_filters(false);

        if self.paths.len() > 1 {
            for path in &self.paths[1..] {
                walker_builder.add(path);
            }
        }
        let walker = walker_builder.build_parallel();

        let (tags_tx, tags_rx) = flume::unbounded();

        let (dir_tx, dir_rx) = flume::unbounded();
        let dir_tx_ = dir_tx.clone();
        let mount = self.mount.clone();
        let paths = self.paths.clone();
        let counter_thread = tokio::task::spawn_blocking(move || {
            for path in paths {
                WalkDir::new(&path)
                    .into_iter()
                    .for_each(|_| dir_tx_.send(Some(DirRead::Found)).unwrap());
            }
        });

        let walker_thread = tokio::task::spawn_blocking(move || {
            walker.run(|| {
                let tags_tx = tags_tx.clone();
                let dir_tx = dir_tx.clone();
                let mount = mount.clone();
                Box::new(move |result| {
                    dir_tx.send(Some(DirRead::Completed)).unwrap();
                    if let Ok(result) = result {
                        let file_path = result.into_path();
                        if file_path.is_file() {
                            if let Ok(Some(metadata)) = SyncEngine::parse_metadata(&file_path) {
                                let file_path_str = clean_file_path(&file_path, &mount);
                                tags_tx
                                    .send(Some((metadata, file_path_str, file_path)))
                                    .map_err(|e| {
                                        SyncError::AsyncError(format!("Error sending tag: {:?}", e))
                                    })
                                    .unwrap();
                            }
                        }
                    }
                    ignore::WalkState::Continue
                })
            });
            dir_tx.send(None).unwrap();
        });

        let tags_handle = self.tags_task(tags_rx);

        self.progress_loop(dir_rx).await;
        walker_thread.await.unwrap();

        counter_thread.await.unwrap();
        match tags_handle.await {
            Ok(Err(e)) => self.tx.send_error(e),
            Ok(Ok(())) => {}
            Err(e) => {
                self.tx.send_error(SyncError::AsyncError(format!(
                    "Error joining tags handle {:?}",
                    e
                )));
            }
        }

        if let Err(e) = self.tx.send(None) {
            warn!("Error sending message to clients {:?}", e);
        }
    }

    async fn progress_loop(&self, dir_rx: flume::Receiver<Option<DirRead>>) {
        let mut processed = 0.0;
        let mut file_count = 0.0;
        while let Ok(Some(dir_read)) = dir_rx.recv_async().await {
            match dir_read {
                DirRead::Found => file_count += 1.0,
                DirRead::Completed => processed += 1.0,
            }
            if file_count > 0.0 && processed <= file_count {
                self.tx
                    .send(Some(Ok(processed / file_count)))
                    .unwrap_or_default();
            }
        }
    }

    fn tags_task(
        &self,
        tags_rx: flume::Receiver<Option<(ReadOnlyTrack, String, PathBuf)>>,
    ) -> JoinHandle<Result<(), SyncError>> {
        let pool = self.pool.clone();
        let cleaned_paths = self
            .paths
            .iter()
            .map(|p| clean_file_path(p, &self.mount))
            .collect();
        tokio::spawn(async move {
            let mut dal = SyncDAL::try_new(pool).await?;
            while let Ok(Some((metadata, path_str, path))) = tags_rx.recv_async().await {
                let mut hasher = DefaultHasher::new();
                metadata.hash(&mut hasher);

                let file_size = path
                    .metadata()
                    .map_err(|e| SyncError::IOError(format!("{:?}", e)))?
                    .len();
                file_size.hash(&mut hasher);
                let fingerprint = hasher.finish().to_string();

                dal.add_artist(&metadata.artist).await?;
                dal.add_album_artist(&metadata.album_artists).await?;
                dal.add_album(&metadata.album, &metadata.album_artists)
                    .await?;

                dal.sync_song(&path_str, &metadata, file_size as i64, &fingerprint)
                    .await?;
            }

            dal.update_missing_songs(cleaned_paths).await?;

            dal.sync_spellfix().await?;
            SyncEngine::add_search_aliases(&mut dal).await?;

            info!("Committing changes");
            dal.commit().await?;
            info!("Finished committing");

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

    fn parse_metadata(file_path: &Path) -> Result<Option<ReadOnlyTrack>, SyncError> {
        let name = file_path.extension().unwrap_or_default();
        let _size = file_path
            .metadata()
            .map_err(|e| SyncError::IOError(format!("{:?}", e)))?
            .len();
        let mut song_metadata: Option<ReadOnlyTrack> = None;
        match &name.to_str().unwrap_or_default().to_lowercase()[..] {
            "mp3" | "m4a" | "ogg" | "wav" | "flac" | "aac" => {
                let tag_result = ReadOnlyTrack::from_path(file_path, None);
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
}
