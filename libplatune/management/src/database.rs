use itertools::Itertools;
use katatsuki::Track;
use libsqlite3_sys::{sqlite3, sqlite3_load_extension};
use log::LevelFilter;
use regex::Regex;
use sqlx::{
    pool::PoolConnection, sqlite::SqliteConnectOptions, ConnectOptions, Pool, Row, Sqlite,
    SqliteConnection, SqlitePool,
};
use std::{
    cmp::Ordering,
    collections::HashMap,
    ffi::{CStr, CString},
    iter::Sum,
    os::raw::c_char,
    ptr,
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tokio::{sync::mpsc, task::JoinHandle, time::timeout};

pub struct Database {
    pool: Pool<Sqlite>,
    opts: SqliteConnectOptions,
}

pub struct SearchOptions<'a> {
    pub start_highlight: &'a str,
    pub end_highlight: &'a str,
    pub limit: i32,
    pub restrict_entry_type: Vec<&'a str>,
}

impl<'a> Default for SearchOptions<'a> {
    fn default() -> Self {
        Self {
            start_highlight: "",
            end_highlight: "",
            limit: 10,
            restrict_entry_type: vec![],
        }
    }
}

#[derive(Debug)]
pub struct SearchRes {
    pub entry: String,
    pub entry_type: String,
    pub description: String,
    pub artist: Option<String>,
    pub correlation_ids: Vec<i32>,
}

#[derive(Debug)]
struct ResultScore {
    len_score: usize,
    weighted_score: f32,
}

impl Sum for ResultScore {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|a, b| ResultScore {
            weighted_score: a.weighted_score + b.weighted_score,
            len_score: a.len_score + b.len_score,
        })
        .unwrap()
    }
}

impl PartialEq for ResultScore {
    fn eq(&self, other: &Self) -> bool {
        self.weighted_score.partial_cmp(&other.weighted_score) == Some(Ordering::Equal)
            && self.len_score.cmp(&other.len_score) == Ordering::Equal
    }
}

impl PartialOrd for ResultScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let ord = self.weighted_score.partial_cmp(&other.weighted_score);
        match ord {
            Some(Ordering::Greater | Ordering::Less) => ord,
            _ => Some(self.len_score.cmp(&other.len_score)),
        }
    }
}

#[derive(Debug)]
struct SearchEntry {
    entry: String,
    pub entry_type: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub correlation_id: i32,
    start_highlight: String,
    end_highlight: String,
    weights: HashMap<String, f32>,
}

impl SearchEntry {
    fn score_match(&self) -> ResultScore {
        let count = self.entry.matches(r"{startmatch}").count();
        let re = Regex::new(&r"(?:\{startmatch\}(.*?)\{endmatch\}[^\s]*).*".repeat(count)).unwrap();
        let caps = re.captures(&self.entry).unwrap();

        let score = caps
            .iter()
            .skip(1)
            .map(|c| match c.map(|c| c.as_str()) {
                Some(cap) => match self.weights.get(cap) {
                    Some(weight) => ResultScore {
                        weighted_score: *weight,
                        len_score: cap.len(),
                    },
                    None => ResultScore {
                        weighted_score: 0.,
                        len_score: cap.len(),
                    },
                },
                None => ResultScore {
                    weighted_score: 0.,
                    len_score: 0,
                },
            })
            .sum();

        return score;
    }

    pub fn get_description(&self) -> String {
        match &self.entry_type[..] {
            "song" => format!(
                "Song from {} by {}",
                self.album.to_owned().unwrap(),
                self.artist.to_owned().unwrap(),
            ),
            "album" => format!("Album by {}", self.artist.to_owned().unwrap()),
            "artist" => "Artist".to_owned(),
            "album_artist" => "Album Artist".to_owned(),
            _ => "".to_owned(),
        }
    }

    pub fn get_formatted_entry(&self) -> String {
        self.entry
            .replace("{startmatch}", &self.start_highlight)
            .replace("{endmatch}", &self.end_highlight)
    }
}

impl Ord for SearchEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for SearchEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_score = self.score_match();
        let other_score = other.score_match();

        self_score.partial_cmp(&other_score)
    }
}

impl Eq for SearchEntry {}

impl PartialEq for SearchEntry {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SpellfixRes {
    word: String,
    search: String,
    score: f32,
}

#[cfg(not(unix))]
fn path_to_cstring<P: AsRef<Path>>(p: &P) -> CString {
    let s = p.as_ref().to_str().unwrap();
    CString::new(s).unwrap()
}

#[cfg(unix)]
fn path_to_cstring<P: AsRef<Path>>(p: &P) -> CString {
    use std::os::unix::ffi::OsStrExt;
    CString::new(p.as_ref().as_os_str().as_bytes()).unwrap()
}

unsafe fn errmsg_to_string(errmsg: *const c_char) -> String {
    let c_slice = CStr::from_ptr(errmsg).to_bytes();
    String::from_utf8_lossy(c_slice).into_owned()
}

fn load_extension<P: AsRef<Path>>(db: *mut sqlite3, dylib_path: &P) {
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

    async fn run_search(
        &self,
        query: &str,
        weights: HashMap<String, f32>,
        options: &SearchOptions<'_>,
        artist_filter: &Vec<String>,
        con: &mut PoolConnection<Sqlite>,
    ) -> Vec<SearchEntry> {
        let artist_select = "CASE entry_type WHEN 'song' THEN ar.artist_name WHEN 'album' THEN aa.album_artist_name ELSE NULL END";
        let order_clause = "rank * (CASE entry_type WHEN 'artist' THEN 1.4 WHEN 'album_artist' THEN 1.4 WHEN 'tag' THEN 1.3 WHEN 'album' THEN 1.25 ELSE 1 END)";
        let mut artist_filter_clause = "".to_owned();
        let num_base_args = 5;
        let mut num_extra_args = 0;
        if artist_filter.len() > 0 {
            let start = num_base_args + num_extra_args + 1;
            let artist_list = (start..start + artist_filter.len())
                .map(|i| "$".to_owned() + &i.to_string())
                .collect_vec()
                .join(",");
            artist_filter_clause = format!("WHERE {} in ({})", artist_select, artist_list);
            num_extra_args += artist_filter.len();
        }
        let mut type_filter = "".to_owned();
        if !options.restrict_entry_type.is_empty() {
            let start = num_base_args + num_extra_args + 1;
            let in_list = (start..start + options.restrict_entry_type.len())
                .map(|i| "$".to_owned() + &i.to_string())
                .collect_vec()
                .join(",");

            type_filter = format!("AND entry_type in ({})", &in_list);
        }
        let full_query = format!("
        WITH CTE AS (
            SELECT DISTINCT entry, entry_type, rank, $1 start_highlight, $2 end_highlight, assoc_id correlation_id,
            {0} artist,
            al2.album_name album,
            ROW_NUMBER() OVER (PARTITION BY 
                entry_value, 
                {0}, 
                CASE entry_type WHEN 'song' THEN 1 WHEN 'album' THEN 2 WHEN 'tag' THEN 3 ELSE 4 END,
                CASE entry_type WHEN 'song' THEN s.song_title + s.album_id WHEN 'album' THEN al.album_name WHEN 'artist' THEN ar2.artist_name WHEN 'album_artist' THEN aa2.album_artist_name END
                ORDER BY entry_type DESC) row_num
            FROM (select entry_type, assoc_id, entry_value, highlight(search_index, 0, '{{startmatch}}', '{{endmatch}}') entry, rank from search_index where entry_value match $3 {3}) a
            LEFT OUTER JOIN song s on s.song_id = assoc_id
            LEFT OUTER JOIN artist ar on ar.artist_id = s.artist_id
            LEFT OUTER JOIN album al on al.album_id = assoc_id
            LEFT OUTER JOIN album al2 on al2.album_id = s.album_id
            LEFT OUTER JOIN album_artist aa on aa.album_artist_id = al.album_artist_id
            LEFT OUTER JOIN artist ar2 on ar2.artist_id = assoc_id
            LEFT OUTER JOIN album_artist aa2 on aa2.album_artist_id = assoc_id
            {2}
            ORDER BY {1}
            LIMIT $4
        )
        SELECT entry, entry_type, artist, album, correlation_id, start_highlight, end_highlight FROM cte
        WHERE row_num = 1
        ORDER BY {1}
        LIMIT $5", artist_select, order_clause, artist_filter_clause, type_filter);
        let mut sql_query = sqlx::query(&full_query)
            .bind(options.start_highlight)
            .bind(options.end_highlight)
            .bind(query.to_owned() + "*")
            .bind(options.limit * 2)
            .bind(options.limit);

        for artist in artist_filter {
            sql_query = sql_query.bind(artist.to_owned());
        }
        for entry_type in &options.restrict_entry_type {
            sql_query = sql_query.bind(entry_type.to_owned());
        }
        let res = sql_query
            .map(|row| SearchEntry {
                entry: row.try_get("entry").unwrap(),
                entry_type: row.try_get("entry_type").unwrap(),
                artist: row.try_get("artist").unwrap(),
                album: row.try_get("album").unwrap(),
                correlation_id: row.try_get("correlation_id").unwrap(),
                start_highlight: row.try_get("start_highlight").unwrap(),
                end_highlight: row.try_get("end_highlight").unwrap(),
                weights: weights.clone(),
            })
            .fetch_all(con)
            .await
            .unwrap();

        return res;
    }

    fn replace_ampersand(&self, string: &str) -> String {
        string.replace(" & ", " and ").replace("&", " ")
    }

    fn convert_res(&self, res: Vec<SearchEntry>) -> Vec<SearchRes> {
        let grouped = res
            .into_iter()
            .group_by(|key| (key.get_formatted_entry(), key.get_description()))
            .into_iter()
            .map(|(key, group)| {
                let group = group.collect_vec();
                let first = group.get(0).unwrap();
                SearchRes {
                    entry: key.0,
                    entry_type: first.entry_type.to_owned(),
                    artist: first.artist.to_owned(),
                    description: key.1,
                    correlation_ids: group.iter().map(|v| v.correlation_id).collect(),
                }
            })
            .collect_vec();

        return grouped;
    }

    async fn search_helper(
        &self,
        query: &str,
        options: SearchOptions<'_>,
        artist_filter: Vec<String>,
    ) -> Vec<SearchRes> {
        let special_chars = Regex::new(r"[^A-Za-z0-9&\s]").unwrap();
        let query = special_chars.replace_all(&query, " ").trim().to_string();
        if query.is_empty() {
            return vec![];
        }
        println!("query {}", query);
        let mut con = self.acquire_with_spellfix().await;
        let mut res = self
            .run_search(
                &(self.replace_ampersand(&query)),
                HashMap::new(),
                &options,
                &artist_filter,
                &mut con,
            )
            .await;
        res.sort();

        if res.len() == options.limit as usize {
            return self.convert_res(res);
        }
        let re = Regex::new(r"\s+").unwrap();
        let terms = re.split(&query).collect_vec();
        let last = terms.last().unwrap().to_owned().to_owned();
        let spellfix_query = terms
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let score_clause = format!("case 
                when word like '% %' then (distance * 1.0 / (length(word) - length(replace(word, ' ', '')))) * 3.5 
                else editdist3(${0}, word) * 1.0 / length(word) end", i + 1);
                format!(
                    "
                    select * from (
                        select distinct word, ${0} search, {1} score
                        from search_spellfix 
                        where word match ${0}
                        and ({1}) <= 50
                        order by {1}
                        limit 5
                    )
                    ",
                    i + 1, score_clause
                )
            })
            .collect_vec()
            .join(" union all ");

        let mut corrected = sqlx::query_as::<_, SpellfixRes>(&spellfix_query);
        for term in terms {
            corrected = corrected.bind(term);
        }
        let mut spellfix_res = corrected.fetch_all(&mut con).await.unwrap();

        spellfix_res.push(SpellfixRes {
            word: last.to_owned(),
            search: last,
            score: 0.,
        });
        let weights = spellfix_res
            .iter()
            .map(|s| (s.word.to_owned(), s.score))
            .collect::<HashMap<_, _>>();
        let mut corrected_search = spellfix_res
            .into_iter()
            .group_by(|row| row.search.to_owned())
            .into_iter()
            .map(|(_, val)| val.map(|v| v.word + " ").collect_vec())
            .fold(vec!["".to_owned()], |a, b| {
                a.into_iter()
                    .flat_map(|x| b.iter().map(move |y| x.clone() + &y))
                    .collect_vec()
            })
            .iter()
            .map(|s| self.replace_ampersand(s))
            .unique()
            .join("OR ")
            .trim()
            .to_owned();
        if corrected_search.is_empty() {
            return vec![];
        }
        corrected_search = special_chars
            .replace_all(&corrected_search, " ")
            .to_string();
        println!("{:?}", corrected_search);

        let rest = self
            .run_search(
                &corrected_search,
                weights.clone(),
                &options,
                &artist_filter,
                &mut con,
            )
            .await;

        for mut r in &mut res {
            r.weights = weights.clone();
        }
        res.extend(rest);
        let mut res = res
            .into_iter()
            .unique_by(|r| {
                r.entry
                    .clone()
                    .replace("{startmatch}", "")
                    .replace("{endmatch}", "")
                    + "-"
                    + &r.entry_type
                    + &r.correlation_id.to_string()
            })
            .take(options.limit as usize)
            .collect_vec();
        res.sort();

        return self.convert_res(res);
    }

    pub async fn search(&self, query: &str, options: SearchOptions<'_>) -> Vec<SearchRes> {
        let mut query = query.to_owned();
        let artist_split = query.split("artist:").collect_vec();
        let mut artist_filter: Vec<String> = vec![];
        if artist_split.len() > 1 {
            let _query = artist_split.get(0).unwrap().to_string();
            let _artist_filter = artist_split.get(1).unwrap().to_string();

            artist_filter = self
                .search_helper(
                    &_artist_filter,
                    SearchOptions {
                        restrict_entry_type: vec!["artist", "album_artist"],
                        ..Default::default()
                    },
                    vec![],
                )
                .await
                .into_iter()
                .map(|r| r.entry)
                .collect_vec();

            query = _query;
        }

        return self.search_helper(&query, options, artist_filter).await;
    }

    pub(crate) async fn sync(&self, folders: Vec<String>) -> tokio::sync::mpsc::Receiver<f32> {
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        if !folders.is_empty() {
            let pool = self.pool.clone();

            tokio::task::spawn_blocking(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(controller(folders, pool, tx));
            });
        }

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
            where length(entry_value) >= 20
            and entry_type != 'song'
            "#
        )
        .fetch_all(&mut tran)
        .await
        .unwrap();
        let re = Regex::new(r"[\s-]+").unwrap();
        for val in long_vals {
            let entry_value = val.entry_value.unwrap();
            let words = re.split(&entry_value).collect_vec();
            if words.len() < 3 {
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

fn load_spellfix(con: &mut SqliteConnection) {
    let handle = con.as_raw_handle();
    #[cfg(target_os = "linux")]
    let path = "./assets/linux/spellfix.o";
    #[cfg(target_os = "windows")]
    let path = "./assets/windows/spellfix.dll";
    load_extension(handle, &Path::new(path));
}

fn spawn_task(
    dispatch_tx: async_channel::Sender<Option<PathBuf>>,
    dispatch_rx: async_channel::Receiver<Option<PathBuf>>,
    finished_tx: mpsc::Sender<DirRead>,
    tags_tx: mpsc::Sender<Option<(Track, String)>>,
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
