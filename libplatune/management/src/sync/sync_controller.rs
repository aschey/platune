use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

use itertools::Itertools;
use katatsuki::Track;
use regex::Regex;
use sqlx::{Pool, Sqlite};
use tokio::{sync::mpsc, task::JoinHandle, time::timeout};

use crate::{
    consts::{MIN_LEN, MIN_WORDS},
    spellfix::load_spellfix,
};

use super::dir_read::DirRead;

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
    let mut tran = pool.begin().await.unwrap();
    load_spellfix(&mut tran);

    tokio::spawn(async move {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        while let Some(metadata) = tags_rx.recv().await {
            match metadata {
                Some((metadata, path)) => {
                    let artist = metadata.artist.trim();
                    let album = metadata.album.trim();
                    let title = metadata.title.trim();
                    let fingerprint = artist.to_owned() + album + title;
                    let _ = sqlx::query!(
                        "insert or ignore into artist(artist_name, created_date) values(?, ?);",
                        artist,
                        timestamp
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
                    "insert or ignore into album_artist(album_artist_name, created_date) values(?, ?);",
                    album_artist,
                    timestamp
                )
                .execute(&mut tran)
                .await
                .unwrap();
                    let _ =
                    sqlx::query!("
                        insert or ignore into album(album_name, album_artist_id, created_date) 
                        values(?, (select album_artist_id from album_artist where album_artist_name = ?), ?);", 
                        album,
                        album_artist,
                        timestamp)
                        .fetch_all(&mut tran)
                        .await
                        .unwrap();

                    let _ = sqlx::query!(
                        "
                    insert into song(
                        song_path,
                        modified_date,
                        created_date,
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
                            ?, ?, ?, ?,
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

        sqlx::query!("select * from song where last_scanned_date < ?", timestamp)
            .fetch_all(&mut tran)
            .await
            .unwrap();

        sqlx::query(
            "
    insert into search_spellfix(word)
    select term
    from search_vocab
    where term not in (
        select word
        from search_spellfix
    )
    ",
        )
        .execute(&mut tran)
        .await
        .unwrap();

        sqlx::query(
            "
    delete from search_spellfix
    where word NOT IN (
        select term
        from search_vocab
    )
    ",
        )
        .execute(&mut tran)
        .await
        .unwrap();

        let long_vals = sqlx::query!(
            r#"
        select entry_value as "entry_value: String"
        from search_index
        where length(entry_value) >= $1
        and entry_type != 'song'
        "#,
            MIN_LEN as i32
        )
        .fetch_all(&mut tran)
        .await
        .unwrap();
        let re = Regex::new(r"[\s-]+").unwrap();
        for val in long_vals {
            let entry_value = val.entry_value.unwrap();
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
            sqlx::query(
                "
                insert into search_spellfix(word, soundslike)
                select $1, $2
                where not exists (
                    select 1 from search_spellfix where word = $1
                )
            ",
            )
            .bind(entry_value.clone())
            .bind(acronym.clone())
            .execute(&mut tran)
            .await
            .unwrap();

            if acronym.contains("&") {
                let replaced = acronym.replace("&", "a");
                sqlx::query(
                    "
                    insert into search_spellfix(word, soundslike)
                    select $1, $2
                    where  (
                        select count(1) from search_spellfix where word = $1
                    ) < 2
                ",
                )
                .bind(entry_value)
                .bind(replaced)
                .execute(&mut tran)
                .await
                .unwrap();
            }
        }

        println!("committing");
        tran.commit().await.unwrap();
        println!("done");
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
