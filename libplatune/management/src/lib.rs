use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use postage::{
    dispatch::{self, Receiver, Sender},
    mpsc,
    prelude::Stream,
    sink::Sink,
};
use sqlx::{Execute, Sqlite, SqlitePool};
use tokio::{task::JoinHandle, time::timeout};

pub async fn traverse() {
    controller("C:\\shared_files\\Music".to_owned()).await;
}

async fn controller(path: String) {
    let mut num_tasks = 1;
    let max_tasks = 100;
    let (mut dispatch_tx, _) = dispatch::channel(10000);
    let (finished_tx, mut finished_rx) = mpsc::channel(10000);
    let (mut tags_tx, tags_rx) = mpsc::channel(10000);
    let tags_handle = tags_task(tags_rx);
    let mut handles = vec![];
    for _ in 0..num_tasks {
        handles.push(spawn_task(
            dispatch_tx.clone(),
            dispatch_tx.subscribe(),
            finished_tx.clone(),
            tags_tx.clone(),
        ));
    }
    dispatch_tx.send(Some(PathBuf::from(path))).await.unwrap();
    let mut dirs = 0;
    loop {
        match timeout(Duration::from_millis(1), finished_rx.recv()).await {
            Ok(Some(DirRead::Completed)) => {
                dirs -= 1;

                if dirs == 0 {
                    break;
                }
            }
            Ok(Some(DirRead::Found)) => {
                dirs += 1;
            }
            Ok(None) => {
                break;
            }
            Err(_) => {
                if num_tasks < max_tasks {
                    println!("spawning task");
                    handles.push(spawn_task(
                        dispatch_tx.clone(),
                        dispatch_tx.subscribe(),
                        finished_tx.clone(),
                        tags_tx.clone(),
                    ));
                    num_tasks += 1;
                }
            }
        }
    }
    tags_tx.send(None).await.unwrap();
    for _ in 0..handles.len() {
        dispatch_tx.send(None).await.unwrap();
    }
    for handle in handles {
        handle.await.unwrap();
    }

    tags_handle.await.unwrap();
}

fn tags_task(mut tags_rx: mpsc::Receiver<Option<SongMetadata>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        dotenv::from_path("./.env").unwrap_or_default();
        let pool = SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        let mut tran = pool.begin().await.unwrap();

        while let Some(metadata) = tags_rx.recv().await {
            match metadata {
                Some(metadata) => {
                    let res = sqlx::query!(
                        "insert or ignore into artist(artist_name) values(?)",
                        metadata.artist
                    )
                    .persistent(true)
                    .execute(&mut tran)
                    .await;
                }
                None => {
                    break;
                }
            }
        }
        println!("committing");
        tran.commit().await.unwrap();
        println!("done");
    })
}

fn spawn_task(
    mut dispatch_tx: Sender<Option<PathBuf>>,
    mut dispatch_rx: Receiver<Option<PathBuf>>,
    mut finished_tx: mpsc::Sender<DirRead>,
    mut tags_tx: mpsc::Sender<Option<SongMetadata>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(path) = dispatch_rx.recv().await {
            match path {
                Some(path) => {
                    for dir_result in path.read_dir().unwrap() {
                        let dir = dir_result.unwrap();

                        if dir.file_type().unwrap().is_file() {
                            let name = dir.path();
                            let name = name.extension().unwrap_or_default();
                            let mut song_metadata: Option<SongMetadata> = None;
                            match name.to_str().unwrap_or_default() {
                                "mp3" | "m4a" => {
                                    let tag_result = katatsuki::Track::from_path(&dir.path(), None);
                                    match tag_result {
                                        Err(e) => {
                                            println!("{:?}", e);
                                        }
                                        Ok(tag) => {
                                            song_metadata = Some(SongMetadata {
                                                artist: Some(tag.artist),
                                                song: Some(tag.title),
                                            });
                                        }
                                    }
                                }

                                _ => {}
                            }
                            if let Some(metadata) = song_metadata {
                                tags_tx.send(Some(metadata)).await.unwrap();
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

#[derive(Debug)]
enum DirRead {
    Found,
    Completed,
}

#[derive(Debug)]
struct SongMetadata {
    artist: Option<String>,
    song: Option<String>,
}
