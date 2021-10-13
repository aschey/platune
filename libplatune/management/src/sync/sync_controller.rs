use std::{path::PathBuf, time::Duration};

use itertools::Itertools;
use katatsuki::Track;
use log::info;
use regex::Regex;
use sqlx::{Pool, Sqlite};
use tokio::{sync::mpsc, task::JoinHandle, time::timeout};

use crate::consts::MIN_WORDS;

use super::{dir_read::DirRead, sync_dal::SyncDAL};

#[derive(Clone)]
pub(crate) struct SyncController {
    pool: Pool<Sqlite>,
}

impl SyncController {
    pub(crate) fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
    pub(crate) async fn sync(
        &self,
        folders: Vec<String>,
        mount: Option<String>,
    ) -> tokio::sync::mpsc::Receiver<f32> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        if !folders.is_empty() {
            let pool = self.pool.clone();

            tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(controller(folders, pool, tx, mount));
            });
        }

        rx
    }
}
async fn controller(
    paths: Vec<String>,
    pool: Pool<Sqlite>,
    tx: mpsc::Sender<f32>,
    mount: Option<String>,
) {
    let mut num_tasks = 1;
    let max_tasks = 100;
    let (dispatch_tx, dispatch_rx) = async_channel::bounded(100);
    let (finished_tx, mut finished_rx) = mpsc::channel(100);
    let (tags_tx, tags_rx) = mpsc::channel(100);
    let tags_handle = tags_task(pool, tags_rx).await;
    let mut handles = vec![];
    for _ in 0..num_tasks {
        handles.push(spawn_task(
            dispatch_tx.clone(),
            dispatch_rx.clone(),
            finished_tx.clone(),
            tags_tx.clone(),
            mount.clone(),
        ));
    }
    for path in paths {
        dispatch_tx.send(Some(PathBuf::from(path))).await.unwrap();
    }

    let mut total_dirs = 0.;
    let mut dirs_processed = 0.;
    loop {
        match timeout(Duration::from_millis(1), finished_rx.recv()).await {
            Ok(Some(DirRead::Completed)) => {
                dirs_processed += 1.;

                // edge case - entire dir is empty
                if total_dirs == 0. {
                    tx.send(1.).await.unwrap();
                    break;
                }
                tx.send(dirs_processed / total_dirs).await.unwrap();

                if total_dirs == dirs_processed {
                    break;
                }
            }
            Ok(Some(DirRead::Found)) => {
                total_dirs += 1.;
                tx.send(dirs_processed / total_dirs).await.unwrap();
            }
            Ok(None) => {
                break;
            }
            Err(_) => {
                if num_tasks < max_tasks {
                    println!("spawning task");
                    handles.push(spawn_task(
                        dispatch_tx.clone(),
                        dispatch_rx.clone(),
                        finished_tx.clone(),
                        tags_tx.clone(),
                        mount.clone(),
                    ));
                    num_tasks += 1;
                }
            }
        }
    }

    for _ in 0..handles.len() {
        dispatch_tx.send(None).await.unwrap();
    }
    for handle in handles {
        handle.await.unwrap();
    }
    tags_tx.send(None).await.unwrap();
    tags_handle.await.unwrap();
}

async fn tags_task(
    pool: Pool<Sqlite>,
    mut tags_rx: mpsc::Receiver<Option<(Track, String)>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut dal = SyncDAL::new(pool).await;

        while let Some(metadata) = tags_rx.recv().await {
            match metadata {
                Some((metadata, path)) => {
                    let fingerprint = metadata.artist.clone() + &metadata.album + &metadata.title;
                    dal.add_artist(&metadata.artist).await;

                    dal.add_album_artist(&metadata.album_artists).await;

                    dal.add_album(&metadata.album, &metadata.album_artists)
                        .await;
                    dal.sync_song(&path, &metadata, &fingerprint).await;
                }
                None => {
                    break;
                }
            }
        }

        // TODO: delete missing songs
        dal.get_missing_songs().await;

        dal.sync_spellfix().await;

        let long_vals = dal.get_long_entries().await;

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
                        w.chars().next().unwrap().to_string().to_lowercase()
                    }
                })
                .collect_vec()
                .join("");

            dal.insert_alias(&entry_value, &acronym).await;
        }

        info!("committing");
        dal.commit().await;
        info!("done");
    })
}

fn spawn_task(
    dispatch_tx: async_channel::Sender<Option<PathBuf>>,
    dispatch_rx: async_channel::Receiver<Option<PathBuf>>,
    finished_tx: mpsc::Sender<DirRead>,
    tags_tx: mpsc::Sender<Option<(Track, String)>>,
    mount: Option<String>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(path) = dispatch_rx.recv().await {
            match path {
                Some(path) => {
                    for dir_result in path.read_dir().unwrap() {
                        let dir = dir_result.unwrap();

                        if dir.file_type().unwrap().is_file() {
                            let file_path = dir.path();
                            let name = file_path.extension().unwrap_or_default();
                            let _size = file_path.metadata().unwrap().len();
                            let mut song_metadata: Option<Track> = None;
                            match &name.to_str().unwrap_or_default().to_lowercase()[..] {
                                "mp3" | "m4a" | "ogg" | "wav" | "flac" | "aac" => {
                                    let tag_result = Track::from_path(&file_path, None);
                                    match tag_result {
                                        Err(e) => {
                                            println!("{:?}", e);
                                        }
                                        Ok(tag) => {
                                            song_metadata = Some(tag);
                                        }
                                    }
                                }

                                _ => {}
                            }
                            if let Some(metadata) = song_metadata {
                                let mut file_path_str = file_path.to_str().unwrap().to_owned();
                                if cfg!(windows) {
                                    file_path_str = file_path_str.replace(r"\", r"/");
                                }
                                if let Some(ref mount) = mount {
                                    if file_path_str.starts_with(&mount[..]) {
                                        file_path_str = file_path_str.replace(&mount[..], "");
                                    }
                                }
                                tags_tx.send(Some((metadata, file_path_str))).await.unwrap();
                            }
                        } else {
                            dispatch_tx.send(Some(dir.path())).await.unwrap();
                            finished_tx.send(DirRead::Found).await.unwrap();
                        }
                    }
                    finished_tx.send(DirRead::Completed).await.unwrap();
                }
                None => {
                    break;
                }
            }
        }
    })
}
