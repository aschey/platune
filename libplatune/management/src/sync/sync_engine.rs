use super::{dir_read::DirRead, sync_dal::SyncDAL};
use crate::{consts::MIN_WORDS, db_error::DbError, path_util::clean_file_path};
use ignore::{WalkBuilder, WalkState};
use itertools::Itertools;
use katatsuki::ReadOnlyTrack;
use regex::Regex;
use sqlx::{Pool, Sqlite};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    time::Instant,
};
use tap::TapFallible;
use thiserror::Error;
use tokio::{
    sync::{
        broadcast,
        mpsc::{self, Sender},
    },
    task::{spawn_blocking, JoinHandle},
};
use tracing::{error, info};
use walkdir::WalkDir;

#[derive(Error, Debug, Clone)]
pub enum SyncError {
    #[error(transparent)]
    DbError(#[from] DbError),
    #[error("Thread communication error: {0}")]
    ThreadCommError(String),
    #[error("IO Error: {0}")]
    IOError(String),
    #[error("Tag read error: {0}")]
    TagReadError(String),
}

trait SendSyncError {
    fn send_error(&self, err: SyncError);
}

impl SendSyncError for broadcast::Sender<Option<Result<f32, SyncError>>> {
    fn send_error(&self, err: SyncError) {
        error!("{err:?}");
        let _ = self
            .send(Some(Err(err)))
            .tap_err(|e| error!("Error sending broadcast message to clients {e:?}"));
    }
}

pub(crate) struct SyncEngine {
    paths: Vec<String>,
    write_pool: Pool<Sqlite>,
    mount: Option<String>,
    tx: broadcast::Sender<Option<Result<f32, SyncError>>>,
}

impl SyncEngine {
    pub(crate) fn new(
        paths: Vec<String>,
        write_pool: Pool<Sqlite>,
        mount: Option<String>,
        tx: broadcast::Sender<Option<Result<f32, SyncError>>>,
    ) -> Self {
        Self {
            paths,
            write_pool,
            mount,
            tx,
        }
    }

    pub(crate) async fn start(&mut self) {
        let start = Instant::now();
        info!("Starting sync process");

        if self.paths.is_empty() {
            return;
        }

        let (tags_tx, tags_rx) = mpsc::channel(10000);
        let (dir_tx, dir_rx) = mpsc::channel(10000);
        let dir_tx_ = dir_tx.clone();

        let counter_task = self.dir_counter(dir_tx_);
        let tags_task = self.tags_parser(tags_tx, dir_tx);
        let db_task = self.db_updater(tags_rx);

        self.progress_loop(dir_rx).await;

        let _ = tags_task.await.tap_err(|e| {
            self.tx.send_error(SyncError::ThreadCommError(format!(
                "Error joining tags handle {e:?}"
            )))
        });
        match counter_task.await {
            Ok(Err(e)) => self.tx.send_error(e),
            Err(e) => {
                self.tx.send_error(SyncError::ThreadCommError(format!(
                    "Error joining counter handle {e:?}"
                )));
            }
            _ => {}
        }

        match db_task.await {
            Ok(Err(e)) => self.tx.send_error(e),
            Err(e) => {
                self.tx.send_error(SyncError::ThreadCommError(format!(
                    "Error joining db handle {e:?}"
                )));
            }
            _ => {}
        }

        info!("Sync took {:?}", start.elapsed());
    }

    fn dir_counter(&self, dir_tx: Sender<DirRead>) -> JoinHandle<Result<(), SyncError>> {
        let paths = self.paths.clone();
        spawn_blocking::<_, Result<(), SyncError>>(move || {
            for path in paths {
                for _ in WalkDir::new(&path).into_iter() {
                    dir_tx.blocking_send(DirRead::Found).map_err(|e| {
                        SyncError::ThreadCommError(format!(
                            "Error sending directory found message: {e:?}"
                        ))
                    })?;
                }
            }
            Ok(())
        })
    }

    fn tags_parser(
        &self,
        tags_tx: Sender<(ReadOnlyTrack, String, PathBuf)>,
        dir_tx: Sender<DirRead>,
    ) -> JoinHandle<()> {
        let mut walker_builder = WalkBuilder::new(&self.paths[0]);
        let num_cpus = num_cpus::get();
        walker_builder.threads(num_cpus).standard_filters(false);

        if self.paths.len() > 1 {
            for path in &self.paths[1..] {
                walker_builder.add(path);
            }
        }
        let walker = walker_builder.build_parallel();
        let mount = self.mount.clone();

        spawn_blocking(move || {
            walker.run(|| {
                let tags_tx = tags_tx.clone();
                let dir_tx = dir_tx.clone();
                let mount = mount.clone();
                Box::new(move |result| {
                    if dir_tx
                        .blocking_send(DirRead::Completed)
                        .tap_err(|e| error!("Error sending completed dir read: {e:?}"))
                        .is_err()
                    {
                        return WalkState::Quit;
                    }

                    if let Ok(result) = result {
                        let file_path = result.into_path();
                        if file_path.is_file() {
                            if let Ok(Some(metadata)) = SyncEngine::parse_metadata(&file_path)
                                .tap_err(|e| error!("Error parsing tag metadata: {e:?}"))
                            {
                                let file_path_str = clean_file_path(&file_path, &mount);

                                if tags_tx
                                    .blocking_send((metadata, file_path_str, file_path))
                                    .tap_err(|e| error!("Error sending tag: {e:?}"))
                                    .is_err()
                                {
                                    return WalkState::Quit;
                                }
                            }
                        }
                    }
                    WalkState::Continue
                })
            });
        })
    }

    async fn progress_loop(&self, mut dir_rx: mpsc::Receiver<DirRead>) {
        let mut processed = 0.0;
        let mut file_count = 0.0;

        while let Some(dir_read) = dir_rx.recv().await {
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

    fn db_updater(
        &self,
        mut tags_rx: mpsc::Receiver<(ReadOnlyTrack, String, PathBuf)>,
    ) -> JoinHandle<Result<(), SyncError>> {
        let write_pool = self.write_pool.clone();
        let cleaned_paths = self
            .paths
            .iter()
            .map(|p| clean_file_path(p, &self.mount))
            .collect_vec();

        tokio::spawn(async move {
            let mut dal = SyncDAL::try_new(write_pool).await?;
            while let Some((metadata, path_str, path)) = tags_rx.recv().await {
                let mut hasher = DefaultHasher::new();
                metadata.hash(&mut hasher);

                let file_size = path
                    .metadata()
                    .map_err(|e| {
                        SyncError::IOError(format!(
                            "Error getting path metadata for {path:?}: {e:?}"
                        ))
                    })?
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

            for path in cleaned_paths {
                dal.update_missing_songs(path).await?;
            }

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

        let re = Regex::new(r"[\s-]+").expect("regex failed to compile");
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
            .map_err(|e| {
                SyncError::IOError(format!(
                    "Error getting track metadata for {file_path:?}: {e:?}"
                ))
            })?
            .len();

        let name = &name.to_str().unwrap_or_default().to_lowercase()[..];
        match name {
            "mp3" | "m4a" | "ogg" | "wav" | "flac" | "aac" => {
                return Ok(Some(ReadOnlyTrack::from_path(file_path, None).map_err(
                    |e| SyncError::TagReadError(format!("Error reading tag: {e:?}")),
                )?));
            }
            _ => {}
        }

        Ok(None)
    }
}
