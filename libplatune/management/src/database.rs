use crate::path_mut::PathMut;
use crate::search::search_engine::SearchEngine;
use crate::search::search_options::SearchOptions;
use crate::search::search_result::SearchResult;
use crate::spellfix::acquire_with_spellfix;
use crate::sync::progress_stream::ProgressStream;
use crate::sync::sync_controller::SyncController;
use crate::{db_error::DbError, entry_type::EntryType};
use log::LevelFilter;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, Pool, Sqlite, SqlitePool};
use std::sync::Arc;
use std::{path::Path, time::Duration};
use tokio::sync::Mutex;
use tracing::info;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
    search_engine: SearchEngine,
    sync_controller: Arc<Mutex<SyncController>>,
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

#[derive(Debug, sqlx::FromRow)]
pub struct DeletedEntry {
    pub song_id: i64,
    pub song_path: String,
}

impl PathMut for DeletedEntry {
    fn get_path(&self) -> String {
        self.song_path.to_owned()
    }

    fn update_path(&mut self, path: String) {
        self.song_path = path
    }
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
            .map_err(|e| DbError::DbError(format!("{:?}", e)))?;
        Ok(Self {
            search_engine: SearchEngine::new(pool.clone()),
            sync_controller: Arc::new(Mutex::new(SyncController::new(pool.clone()))),
            pool,
        })
    }

    pub async fn migrate(&self) -> Result<(), DbError> {
        let mut con = acquire_with_spellfix(&self.pool).await?;

        info!("Migrating");
        sqlx::migrate!("./migrations")
            .run(&mut con)
            .await
            .map_err(|e| {
                DbError::DbError(format!("Error running migrations {}", format!("{:?}", e)))
            })?;
        info!("Finished migrating");

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
        self.search_engine.search(query, options).await
    }

    pub(crate) async fn sync(
        &mut self,
        folders: Vec<String>,
        mount: Option<String>,
    ) -> ProgressStream {
        let search_engine = self.search_engine.clone();
        self.sync_controller
            .lock()
            .await
            .sync(
                folders,
                mount,
                Box::new(move || search_engine.clear_cache()),
            )
            .await
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
        sqlx::query_as!(
            LookupEntry,
            "
            SELECT ar.artist_name artist, s.song_title song, s.song_path path, 
            al.album_name album, aa.album_artist_name album_artist, s.track_number track
            FROM artist ar
            INNER JOIN song s ON s.artist_id = ar.artist_id
            INNER JOIN album al ON al.album_id = s.album_id
            INNER JOIN album_artist aa ON aa.album_artist_id = al.album_artist_id
            WHERE ar.artist_id = ?
            ORDER BY aa.album_artist_id, al.album_id, s.track_number;
            ",
            artist_ids[0]
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DbError(format!("{:?}", e)))
    }

    async fn all_by_album_artists(
        &self,
        album_artist_ids: Vec<i32>,
    ) -> Result<Vec<LookupEntry>, DbError> {
        sqlx::query_as!(
            LookupEntry,
            "
            SELECT ar.artist_name artist, s.song_title song, s.song_path path, 
            al.album_name album, aa.album_artist_name album_artist, s.track_number track
            FROM album_artist aa
            INNER JOIN album al ON al.album_artist_id = aa.album_artist_id
            INNER JOIN song s ON s.album_id = al.album_id
            INNER JOIN artist ar ON ar.artist_id = s.artist_id
            WHERE aa.album_artist_id = ?
            ORDER BY aa.album_artist_id, al.album_id, s.track_number;",
            album_artist_ids[0]
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DbError(format!("{:?}", e)))
    }

    async fn all_by_albums(&self, album_ids: Vec<i32>) -> Result<Vec<LookupEntry>, DbError> {
        sqlx::query_as!(
            LookupEntry,
            "
            SELECT ar.artist_name artist, s.song_title song, s.song_path path, 
            al.album_name album, aa.album_artist_name album_artist, s.track_number track 
            FROM album al
            INNER JOIN album_artist aa ON aa.album_artist_id = al.album_artist_id
            INNER JOIN song s ON s.album_id = al.album_id
            INNER JOIN artist ar ON ar.artist_id = s.artist_id
            WHERE al.album_id = ?
            ORDER BY aa.album_artist_id, al.album_id, s.track_number;
            ",
            album_ids[0]
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DbError(format!("{:?}", e)))
    }

    async fn all_by_ids(&self, song_ids: Vec<i32>) -> Result<Vec<LookupEntry>, DbError> {
        sqlx::query_as!(
            LookupEntry,
            "
            SELECT ar.artist_name artist, s.song_title song, s.song_path path, 
            al.album_name album, aa.album_artist_name album_artist, s.track_number track
            FROM song s
            INNER JOIN artist ar ON ar.artist_id = s.artist_id
            INNER JOIN album al ON al.album_id = s.album_id
            INNER JOIN album_artist aa ON aa.album_artist_id = al.album_artist_id
            WHERE s.song_id = ?
            ORDER BY aa.album_artist_id, al.album_id, s.track_number;
            ",
            song_ids[0]
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DbError(format!("{:?}", e)))
    }

    pub(crate) async fn get_deleted_songs(&self) -> Result<Vec<DeletedEntry>, DbError> {
        sqlx::query_as!(
            DeletedEntry,
            "
            SELECT ds.song_id, song_path FROM deleted_song ds
            INNER JOIN song s ON s.song_id = ds.song_id;
            "
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DbError::DbError(format!("{:?}", e)))
    }

    pub(crate) async fn delete_tracks(&self, ids: Vec<i64>) -> Result<(), DbError> {
        let mut tran = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::DbError(format!("{:?}", e)))?;
        for id in ids {
            sqlx::query!("DELETE FROM deleted_song WHERE song_id = ?;", id)
                .execute(&mut tran)
                .await
                .map_err(|e| DbError::DbError(format!("{:?}", e)))?;

            sqlx::query!("DELETE FROM song WHERE song_id = ?;", id)
                .execute(&mut tran)
                .await
                .map_err(|e| DbError::DbError(format!("{:?}", e)))?;
        }

        tran.commit()
            .await
            .map_err(|e| DbError::DbError(format!("{:?}", e)))
    }

    pub(crate) async fn add_folders(&self, paths: Vec<String>) -> Result<(), DbError> {
        let mut tran = self
            .pool
            .begin()
            .await
            .map_err(|e| DbError::DbError(format!("{:?}", e)))?;
        for path in paths {
            sqlx::query!("INSERT OR IGNORE INTO folder(folder_path) VALUES(?);", path)
                .execute(&mut tran)
                .await
                .map_err(|e| DbError::DbError(format!("{:?}", e)))?;
        }
        tran.commit()
            .await
            .map_err(|e| DbError::DbError(format!("{:?}", e)))
    }

    pub(crate) async fn update_folder(
        &self,
        old_path: String,
        new_path: String,
    ) -> Result<SqliteQueryResult, DbError> {
        sqlx::query!(
            "UPDATE folder SET folder_path = ? WHERE folder_path = ?;",
            new_path,
            old_path
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DbError(format!("{:?}", e)))
    }

    pub(crate) async fn get_all_folders(&self) -> Result<Vec<String>, DbError> {
        Ok(sqlx::query!("SELECT folder_path FROM folder;")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DbError::DbError(format!("{:?}", e)))?
            .into_iter()
            .map(|r| r.folder_path)
            .collect())
    }

    pub(crate) async fn get_mount(&self, mount_id: i64) -> Option<String> {
        match sqlx::query!("SELECT mount_path FROM mount WHERE mount_id = ?;", mount_id)
            .fetch_one(&self.pool)
            .await
        {
            Ok(res) => Some(res.mount_path),
            Err(_) => None,
        }
    }

    pub(crate) async fn add_mount(&self, path: &str) -> Result<i64, DbError> {
        sqlx::query!(r"INSERT OR IGNORE INTO mount(mount_path) VALUES(?);", path)
            .execute(&self.pool)
            .await
            .map_err(|e| DbError::DbError(format!("{:?}", e)))?;

        let res = sqlx::query!(r"SELECT mount_id FROM mount WHERE mount_path = ?;", path)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| DbError::DbError(format!("{:?}", e)))?;

        Ok(res.mount_id)
    }

    pub(crate) async fn update_mount(&self, mount_id: i64, path: &str) -> Result<u64, DbError> {
        let res = sqlx::query!(
            "UPDATE mount SET mount_path = ? WHERE mount_id = ?;",
            path,
            mount_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DbError::DbError(format!("{:?}", e)))?;

        Ok(res.rows_affected())
    }
}
