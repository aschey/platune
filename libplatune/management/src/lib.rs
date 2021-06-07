use std::{
    os::windows::prelude::MetadataExt,
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};

use katatsuki::Track;
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

fn tags_task(mut tags_rx: mpsc::Receiver<Option<(Track, String)>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        dotenv::from_path("./.env").unwrap_or_default();
        let pool = SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;
        let mut tran = pool.begin().await.unwrap();

        while let Some(metadata) = tags_rx.recv().await {
            match metadata {
                Some((metadata, path)) => {
                    let artist = metadata.artist.trim();
                    let album = metadata.album.trim();
                    let title = metadata.title.trim();
                    let fingerprint = artist.to_owned() + album + title;
                    let _ = sqlx::query!(
                        "insert or ignore into artist(artist_name) values(?);",
                        artist
                    )
                    .execute(&mut tran)
                    .await
                    .unwrap();
                    let album_artist =
                        if metadata.album_artists.len() > 0 && metadata.album_artists[0] != "" {
                            metadata.album_artists.join(",")
                        } else {
                            artist.to_owned()
                        };

                    let _ = sqlx::query!(
                        "insert or ignore into album_artist(album_artist_name) values(?);",
                        album_artist
                    )
                    .execute(&mut tran)
                    .await
                    .unwrap();
                    let _ =
                        sqlx::query!("
                            insert or ignore into album(album_name, album_artist_id) 
                            values(?, (select album_artist_id from album_artist where album_artist_name = ?));", album, album_artist)
                            .fetch_all(&mut tran)
                            .await
                            .unwrap();

                    let _ = sqlx::query!(
                        "
                        insert into song(
                            song_path,
                            modified_date,
                            last_scanned_date,
                            artist_id,
                            song_title,
                            album_id,
                            track_number,
                            disc_number,
                            song_year,
                            song_month,
                            song_day,
                            duration,
                            sample_rate,
                            bit_rate,
                            album_art_path,
                            fingerprint
                            )
                            values
                            (
                                ?, ?, ?,
                                (select artist_id from artist where artist_name = ?), 
                                ?, 
                                (
                                    select album_id from album a
                                    inner join album_artist aa on a.album_artist_id = aa.album_artist_id
                                    where a.album_name = ? and aa.album_artist_name = ?
                                ), 
                                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                            )
                            on conflict(song_path) do update
                            set last_scanned_date = ?;
                        ",
                        path,
                        timestamp,
                        timestamp,
                        artist,
                        title,
                        album,
                        album_artist,
                        metadata.track_number,
                        metadata.disc_number,
                        metadata.year,
                        0,
                        0,
                        metadata.duration,
                        metadata.sample_rate,
                        metadata.bitrate,
                        "",
                        fingerprint,
                        timestamp
                    )
                    .execute(&mut tran)
                    .await
                    .unwrap();

                    let _ = sqlx::query!(
                        "
                        update song
                            set modified_date = $2,
                            artist_id = (select artist_id from artist where artist_name = $3),
                            song_title = $4,
                            album_id = (select album_id from album where album_name = $5),
                            track_number = $6,
                            disc_number = $7,
                            song_year = $8,
                            song_month = $9,
                            song_day = $10,
                            duration = $11,
                            sample_rate = $12,
                            bit_rate = $13,
                            album_art_path = $14,
                            fingerprint = $15
                        where song_path = $1 and fingerprint != $15;
                        ",
                        path,
                        timestamp,
                        artist,
                        title,
                        album,
                        metadata.track_number,
                        metadata.disc_number,
                        metadata.year,
                        0,
                        0,
                        metadata.duration,
                        metadata.sample_rate,
                        metadata.bitrate,
                        "",
                        fingerprint
                    )
                    .execute(&mut tran)
                    .await
                    .unwrap();
                }
                None => {
                    break;
                }
            }
        }
        println!("committing");
        tran.commit().await.unwrap();
        sqlx::query!("select * from song where last_scanned_date < ?", timestamp)
            .fetch_all(&pool)
            .await
            .unwrap();
        println!("done");
    })
}

fn spawn_task(
    mut dispatch_tx: Sender<Option<PathBuf>>,
    mut dispatch_rx: Receiver<Option<PathBuf>>,
    mut finished_tx: mpsc::Sender<DirRead>,
    mut tags_tx: mpsc::Sender<Option<(Track, String)>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(path) = dispatch_rx.recv().await {
            match path {
                Some(path) => {
                    for dir_result in path.read_dir().unwrap() {
                        let dir = dir_result.unwrap();

                        if dir.file_type().unwrap().is_file() {
                            let file_path = dir.path();
                            let name = file_path.extension().unwrap_or_default();
                            let size = file_path.metadata().unwrap().file_size();
                            let mut song_metadata: Option<Track> = None;
                            match &name.to_str().unwrap_or_default().to_lowercase()[..] {
                                "mp3" | "m4a" | "ogg" | "wav" | "flac" | "aac" => {
                                    let tag_result = Track::from_path(&dir.path(), None);
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
                                tags_tx
                                    .send(Some((metadata, file_path.to_str().unwrap().to_owned())))
                                    .await
                                    .unwrap();
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
