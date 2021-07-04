use itertools::Itertools;
use katatsuki::Track;
use libsqlite3_sys::{sqlite3, sqlite3_load_extension};
use log::LevelFilter;
use postage::{
    dispatch::{self, Receiver, Sender},
    mpsc,
    prelude::Stream,
    sink::Sink,
};
use regex::Regex;
use sqlx::{
    migrate::Migrate, pool::PoolConnection, sqlite::SqliteConnectOptions, Acquire, ConnectOptions,
    Connection, Pool, Sqlite, SqliteConnection, SqlitePool, Transaction,
};
use std::{
    env,
    ffi::{CStr, CString},
    ops::{Deref, DerefMut},
    os::raw::c_char,
    ptr,
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tokio::{task::JoinHandle, time::timeout};

pub struct Database {
    pool: Pool<Sqlite>,
    opts: SqliteConnectOptions,
}

#[derive(Debug, sqlx::FromRow)]
pub struct SearchRes {
    formatted_entry: String,
    entry_type: String,
    artist: Option<String>,
    correlation_id: i32,
}

#[derive(Debug, sqlx::FromRow)]
struct SpellFixRes {
    word: String,
    search: String,
}

#[cfg(not(unix))]
fn path_to_cstring(p: &Path) -> CString {
    let s = p.to_str().unwrap();
    CString::new(s).unwrap()
}

#[cfg(unix)]
fn path_to_cstring(p: &Path) -> CString {
    use std::os::unix::ffi::OsStrExt;
    CString::new(p.as_os_str().as_bytes()).unwrap()
}

unsafe fn errmsg_to_string(errmsg: *const c_char) -> String {
    let c_slice = CStr::from_ptr(errmsg).to_bytes();
    String::from_utf8_lossy(c_slice).into_owned()
}

fn load_extension(db: *mut sqlite3, dylib_path: &Path) {
    let dylib_str = path_to_cstring(dylib_path);
    unsafe {
        let mut errmsg: *mut c_char = ptr::null_mut();

        let res = sqlite3_load_extension(db, dylib_str.as_ptr(), ptr::null(), &mut errmsg);
        if res != 0 {
            println!("{}", errmsg_to_string(errmsg));
        }
    }
}

impl Database {
    pub async fn connect(path: impl AsRef<Path>, create_if_missing: bool) -> Self {
        let opts = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .create_if_missing(create_if_missing)
            .log_statements(LevelFilter::Debug)
            .log_slow_statements(LevelFilter::Info, Duration::from_secs(1))
            .to_owned();

        let pool = SqlitePool::connect_with(opts.clone()).await.unwrap();

        Self { pool, opts }
    }

    async fn acquire_with_spellfix(&self) -> PoolConnection<Sqlite> {
        let mut con = self.pool.acquire().await.unwrap();
        load_spellfix(&mut con);
        con
    }

    pub async fn migrate(&self) {
        let mut con = self.acquire_with_spellfix().await;

        sqlx::migrate!("./migrations").run(&mut con).await.unwrap();

        println!("done");
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }

    pub async fn search(&self, query: &str, limit: i32) -> Vec<SearchRes> {
        // blss, bliss, bless
        let mut con = self.acquire_with_spellfix().await;
        let re = Regex::new(r"\s+").unwrap();
        let terms = re.split(query).collect::<Vec<_>>();
        let spellfix_query = terms
            .iter()
            .enumerate()
            .map(|(i, _)| {
                format!(
                    "select word, ${0} as search from search_spellfix 
                    where word match ${0}
                    and distance = (
                        select distance 
                        from search_spellfix 
                        where word match ${0}
                        order by distance 
                        limit 1
                    )",
                    i + 1
                )
            })
            .collect::<Vec<_>>()
            .join(" union all ");

        let mut corrected = sqlx::query_as::<_, SpellFixRes>(&spellfix_query);
        for term in terms {
            corrected = corrected.bind(term);
        }
        let spellfix_res = corrected.fetch_all(&mut con).await.unwrap();
        let corrected_search = spellfix_res
            .into_iter()
            .group_by(|row| row.search.to_owned())
            .into_iter()
            .map(|(_, val)| val.map(|v| v.word + " ").collect::<Vec<_>>())
            .fold(vec!["".to_owned()], |a, b| {
                a.into_iter()
                    .flat_map(|x| b.iter().map(move |y| x.clone() + &y))
                    .collect_vec()
            })
            .join("OR ");

        println!("{:?}", corrected_search);

        let artist_select = "CASE entry_type WHEN 'song' THEN ar.artist_name WHEN 'album' THEN aa.album_artist_name ELSE NULL END";
        let order_clause = "rank * (CASE entry_type WHEN 'artist' THEN 1.4 WHEN 'album_artist' THEN 1.4 WHEN 'tag' THEN 1.3 WHEN 'album' THEN 1.25 ELSE 1 END)";

        let res = sqlx::query_as::<_, SearchRes>(&format!("
        WITH CTE AS (
            SELECT DISTINCT formatted_entry, entry_type, rank,
            CASE entry_type WHEN 'song' THEN ar.artist_id WHEN 'album' THEN al.album_id ELSE assoc_id END correlation_id,
            {0} artist,
            ROW_NUMBER() OVER (PARTITION BY entry_value, {0}, CASE entry_type WHEN 'song' THEN 1 WHEN 'album' THEN 2 WHEN 'tag' THEN 3 ELSE 4 END ORDER BY entry_type DESC) row_num
            FROM (select entry_type, assoc_id, entry_value, highlight(search_index, 0, '{{', '}}') formatted_entry, rank from search_index where entry_value match ?) a
            LEFT OUTER JOIN song s on s.song_id = assoc_id
            LEFT OUTER JOIN artist ar on ar.artist_id = s.artist_id
            LEFT OUTER JOIN album al on al.album_id = assoc_id
            LEFT OUTER JOIN album_artist aa on aa.album_artist_id = al.album_artist_id
            ORDER BY {1}
            LIMIT ?
        )
        SELECT formatted_entry, entry_type, artist, correlation_id FROM cte
        WHERE row_num = 1
        ORDER BY {1}
        LIMIT ?", artist_select, order_clause))
        .bind(corrected_search)
        .bind(limit * 2)
        .bind(limit)
        .fetch_all(&mut con)
        .await
        .unwrap();

        return res;
    }

    pub(crate) async fn sync(&self, folders: Vec<String>) -> tokio::sync::mpsc::Receiver<f32> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let pool = self.pool.clone();

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(controller(folders, pool, tx));
        });

        rx
    }

    pub(crate) async fn add_folders(&self, paths: Vec<String>) {
        let mut tran = self.pool.begin().await.unwrap();
        for path in paths {
            sqlx::query!("insert or ignore into folder(folder_path) values(?)", path)
                .execute(&mut tran)
                .await
                .unwrap();
        }
        tran.commit().await.unwrap();
    }

    pub(crate) async fn update_folder(&self, old_path: String, new_path: String) {
        sqlx::query!(
            "update folder set folder_path = ? where folder_path = ?",
            new_path,
            old_path
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }

    pub(crate) async fn get_all_folders(&self) -> Vec<String> {
        sqlx::query!("select folder_path from folder")
            .fetch_all(&self.pool)
            .await
            .unwrap()
            .into_iter()
            .map(|r| r.folder_path)
            .collect()
    }

    pub(crate) async fn get_mount(&self, mount_id: String) -> Option<String> {
        match sqlx::query!("select mount_path from mount where mount_id = ?", mount_id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(res) => Some(res.mount_path),
            Err(_) => None,
        }
    }

    pub(crate) async fn add_mount(&self, path: &str) -> i32 {
        let res = sqlx::query!(
            r#"insert or ignore into mount(mount_path) values(?) returning mount_id as "mount_id: i32""#,
            path
        )
        .fetch_one(&self.pool)
        .await
        .unwrap();

        return res.mount_id.unwrap();
    }

    pub(crate) async fn update_mount(&self, mount_id: String, path: &str) {
        sqlx::query!(
            "update mount set mount_path = ? where mount_id = ?",
            path,
            mount_id
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            opts: self.opts.clone(),
        }
    }
}

async fn controller(paths: Vec<String>, pool: Pool<Sqlite>, tx: tokio::sync::mpsc::Sender<f32>) {
    let mut num_tasks = 1;
    let max_tasks = 100;
    let (mut dispatch_tx, _) = dispatch::channel(10000);
    let (finished_tx, mut finished_rx) = mpsc::channel(10000);
    let (mut tags_tx, tags_rx) = mpsc::channel(10000);
    let tags_handle = tags_task(pool, tags_rx).await;
    let mut handles = vec![];
    for _ in 0..num_tasks {
        handles.push(spawn_task(
            dispatch_tx.clone(),
            dispatch_tx.subscribe(),
            finished_tx.clone(),
            tags_tx.clone(),
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

        println!("committing");
        tran.commit().await.unwrap();
        println!("done");
    })
}

fn load_spellfix(con: &mut SqliteConnection) {
    let handle = con.as_raw_handle();
    #[cfg(target_os = "linux")]
    let path = "./assets/linux/spellfix.o";
    #[cfg(target_os = "windows")]
    let path = "./assets/windows/spellfix.dll";
    load_extension(handle, &Path::new(path));
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
                            let _size = file_path.metadata().unwrap().len();
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
