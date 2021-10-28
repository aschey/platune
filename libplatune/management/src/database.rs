use crate::search::search_engine::SearchEngine;
use crate::search::search_options::SearchOptions;
use crate::search::search_result::SearchResult;
use crate::spellfix::acquire_with_spellfix;
use crate::sync::sync_controller::SyncController;
use crate::{db_error::DbError, entry_type::EntryType};
use log::LevelFilter;
use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, Pool, Sqlite, SqlitePool};
use std::{path::Path, time::Duration};
use tokio::sync::mpsc::Receiver;
use tracing::info;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
    search_engine: SearchEngine,
    sync_controller: SyncController,
    opts: SqliteConnectOptions,
}

#[derive(Debug, sqlx::FromRow)]
pub struct LookupEntry {
    pub artist: String,
    pub album_artist: String,
    pub album: String,
    pub song: String,
    pub path: String,
    pub track: i64,
}

impl Database {
    pub async fn connect(path: impl AsRef<Path>, create_if_missing: bool) -> Result<Self, DbError> {
        let opts = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .create_if_missing(create_if_missing)
            .log_statements(LevelFilter::Debug)
            .log_slow_statements(LevelFilter::Info, Duration::from_secs(1))
            .to_owned();

        let pool = SqlitePool::connect_with(opts.clone())
            .await
            .map_err(DbError::DbError)?;
        Ok(Self {
            search_engine: SearchEngine::new(pool.clone()),
            sync_controller: SyncController::new(pool.clone()),
            pool,
            opts,
        })
    }

    pub async fn migrate(&self) -> Result<(), DbError> {
        let mut con = acquire_with_spellfix(&self.pool).await?;

        info!("migrating");
        sqlx::migrate!("./migrations")
            .run(&mut con)
            .await
            .map_err(DbError::MigrateError)?;
        info!("done");

        Ok(())
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }

    pub(crate) async fn search(
        &self,
        query: &str,
        options: SearchOptions<'_>,
    ) -> Result<Vec<SearchResult>, DbError> {
        Ok(self.search_engine.search(query, options).await?)
    }

    pub(crate) async fn sync(
        &self,
        folders: Vec<String>,
        mount: Option<String>,
    ) -> Receiver<Result<f32, DbError>> {
        self.sync_controller.sync(folders, mount).await
    }

    pub(crate) async fn lookup(
        &self,
        correlation_ids: Vec<i32>,
        entry_type: EntryType,
    ) -> Result<Vec<LookupEntry>, DbError> {
        match entry_type {
            EntryType::Album => self.all_by_albums(correlation_ids).await,
            EntryType::Song => self.all_by_ids(correlation_ids).await,
            EntryType::Artist => self.all_by_artists(correlation_ids).await,
            EntryType::AlbumArtist => self.all_by_album_artists(correlation_ids).await,
        }
    }

    async fn all_by_artists(&self, artist_ids: Vec<i32>) -> Result<Vec<LookupEntry>, DbError> {
        Ok(sqlx::query_as!(
            LookupEntry,
            "
            select ar.artist_name artist, s.song_title song, s.song_path path, 
            al.album_name album, aa.album_artist_name album_artist, s.track_number track
            from artist ar
            inner join song s on s.artist_id = ar.artist_id
            inner join album al on al.album_id = s.album_id
            inner join album_artist aa on aa.album_artist_id = al.album_artist_id
            where ar.artist_id = ?
            order by aa.album_artist_id, al.album_id, s.track_number",
            artist_ids[0]
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DbError)?)
    }

    async fn all_by_album_artists(
        &self,
        album_artist_ids: Vec<i32>,
    ) -> Result<Vec<LookupEntry>, DbError> {
        Ok(sqlx::query_as!(
            LookupEntry,
            "
            select ar.artist_name artist, s.song_title song, s.song_path path, 
            al.album_name album, aa.album_artist_name album_artist, s.track_number track
            from album_artist aa
            inner join album al on al.album_artist_id = aa.album_artist_id
            inner join song s on s.album_id = al.album_id
            inner join artist ar on ar.artist_id = s.artist_id
            where aa.album_artist_id = ?
            order by aa.album_artist_id, al.album_id, s.track_number",
            album_artist_ids[0]
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DbError)?)
    }

    async fn all_by_albums(&self, album_ids: Vec<i32>) -> Result<Vec<LookupEntry>, DbError> {
        Ok(sqlx::query_as!(
            LookupEntry,
            "
            select ar.artist_name artist, s.song_title song, s.song_path path, 
            al.album_name album, aa.album_artist_name album_artist, s.track_number track 
            from album al
            inner join album_artist aa on aa.album_artist_id = al.album_artist_id
            inner join song s on s.album_id = al.album_id
            inner join artist ar on ar.artist_id = s.artist_id
            where al.album_id = ?
            order by aa.album_artist_id, al.album_id, s.track_number
            ",
            album_ids[0]
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DbError)?)
    }

    async fn all_by_ids(&self, song_ids: Vec<i32>) -> Result<Vec<LookupEntry>, DbError> {
        Ok(sqlx::query_as!(
            LookupEntry,
            "
            select ar.artist_name artist, s.song_title song, s.song_path path, 
            al.album_name album, aa.album_artist_name album_artist, s.track_number track
            from song s
            inner join artist ar on ar.artist_id = s.artist_id
            inner join album al on al.album_id = s.album_id
            inner join album_artist aa on aa.album_artist_id = al.album_artist_id
            where s.song_id = ?
            order by aa.album_artist_id, al.album_id, s.track_number
            ",
            song_ids[0]
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::DbError)?)
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

    pub(crate) async fn get_mount(&self, mount_id: i64) -> Option<String> {
        match sqlx::query!("select mount_path from mount where mount_id = ?", mount_id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(res) => Some(res.mount_path),
            Err(_) => None,
        }
    }

    pub(crate) async fn add_mount(&self, path: &str) -> i64 {
        sqlx::query!(r"insert or ignore into mount(mount_path) values(?)", path)
            .execute(&self.pool)
            .await
            .unwrap();

        let res = sqlx::query!(r"select mount_id from mount where mount_path = ?", path)
            .fetch_one(&self.pool)
            .await
            .unwrap();

        res.mount_id
    }

    pub(crate) async fn update_mount(&self, mount_id: i64, path: &str) -> u64 {
        let res = sqlx::query!(
            "update mount set mount_path = ? where mount_id = ?",
            path,
            mount_id
        )
        .execute(&self.pool)
        .await
        .unwrap();

        res.rows_affected()
    }
}
