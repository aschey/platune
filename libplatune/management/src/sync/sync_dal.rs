use std::time::SystemTime;

use itertools::Itertools;
use katatsuki::Track;
use sqlx::{sqlite::SqliteQueryResult, Pool, Sqlite, Transaction};

use crate::{consts::MIN_LEN, db_error::DbError, spellfix::load_spellfix};

pub(crate) struct SyncDAL<'a> {
    tran: Transaction<'a, Sqlite>,
    timestamp: u32,
}

impl<'a> SyncDAL<'a> {
    pub(crate) async fn try_new(pool: Pool<Sqlite>) -> Result<SyncDAL<'a>, DbError> {
        let mut tran = pool.begin().await.map_err(DbError::DbError)?;
        load_spellfix(&mut tran)?;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        Ok(Self { tran, timestamp })
    }

    pub(crate) async fn add_artist(&mut self, artist: &str) -> Result<SqliteQueryResult, DbError> {
        sqlx::query!(
            "insert or ignore into artist(artist_name, created_date) values(?, ?);",
            artist,
            self.timestamp
        )
        .execute(&mut self.tran)
        .await
        .map_err(DbError::DbError)
    }

    pub(crate) async fn add_album_artist(
        &mut self,
        album_artist: &str,
    ) -> Result<SqliteQueryResult, DbError> {
        sqlx::query!(
            "insert or ignore into album_artist(album_artist_name, created_date) values(?, ?);",
            album_artist,
            self.timestamp
        )
        .execute(&mut self.tran)
        .await
        .map_err(DbError::DbError)
    }

    pub(crate) async fn add_album(
        &mut self,
        album: &str,
        album_artist: &str,
    ) -> Result<SqliteQueryResult, DbError> {
        sqlx::query!(
            "
        insert or ignore into album(album_name, album_artist_id, created_date) 
        values(?, (select album_artist_id from album_artist where album_artist_name = ?), ?);",
            album,
            album_artist,
            self.timestamp
        )
        .execute(&mut self.tran)
        .await
        .map_err(DbError::DbError)
    }

    pub(crate) async fn sync_song(
        &mut self,
        path: &str,
        metadata: &Track,
        fingerprint: &str,
    ) -> Result<SqliteQueryResult, DbError> {
        self.add_song(path, metadata, fingerprint).await?;
        self.update_song(path, metadata, fingerprint).await
    }

    pub(crate) async fn get_missing_songs(&mut self) -> Result<Vec<String>, DbError> {
        let paths = sqlx::query!(
            "select song_path from song where last_scanned_date < ?",
            self.timestamp
        )
        .fetch_all(&mut self.tran)
        .await?
        .into_iter()
        .map(|r| r.song_path)
        .collect_vec();

        Ok(paths)
    }

    pub(crate) async fn sync_spellfix(&mut self) -> Result<(), DbError> {
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
        .execute(&mut self.tran)
        .await
        .map_err(DbError::DbError)?;

        sqlx::query(
            "
            delete from search_spellfix
            where word NOT IN (
                select term
                from search_vocab
            )
            ",
        )
        .execute(&mut self.tran)
        .await
        .map_err(DbError::DbError)?;

        Ok(())
    }

    pub(crate) async fn get_long_entries(&mut self) -> Result<Vec<String>, DbError> {
        let long_vals = sqlx::query!(
            r#"
            select entry_value as "entry_value: String"
            from search_index
            where length(entry_value) >= $1
            and entry_type != 'song'
            "#,
            MIN_LEN as i32
        )
        .fetch_all(&mut self.tran)
        .await
        .map_err(DbError::DbError)?
        .into_iter()
        .map(|r| r.entry_value.unwrap_or_default())
        .collect_vec();

        Ok(long_vals)
    }

    pub(crate) async fn insert_alias(
        &mut self,
        entry_value: &str,
        acronym: &str,
    ) -> Result<(), DbError> {
        sqlx::query(
            "
            insert into search_spellfix(word, soundslike)
            select $1, $2
            where not exists (
                select 1 from search_spellfix where word = $1
            )
        ",
        )
        .bind(entry_value)
        .bind(acronym)
        .execute(&mut self.tran)
        .await
        .map_err(DbError::DbError)?;

        if acronym.contains('&') {
            let replaced = acronym.replace("&", "a");
            self.insert_alt_alias(entry_value, &replaced).await?;
        }

        Ok(())
    }

    pub(crate) async fn commit(self) -> Result<(), DbError> {
        self.tran.commit().await.map_err(DbError::DbError)
    }

    async fn insert_alt_alias(
        &mut self,
        entry_value: &str,
        acronym: &str,
    ) -> Result<SqliteQueryResult, DbError> {
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
        .bind(acronym)
        .execute(&mut self.tran)
        .await
        .map_err(DbError::DbError)
    }

    async fn add_song(
        &mut self,
        path: &str,
        metadata: &Track,
        fingerprint: &str,
    ) -> Result<SqliteQueryResult, DbError> {
        sqlx::query!(
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
            self.timestamp,
            self.timestamp,
            self.timestamp,
            metadata.artist,
            metadata.title,
            metadata.album,
            metadata.album_artists,
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
            self.timestamp
        )
        .execute(&mut self.tran)
        .await
        .map_err(DbError::DbError)
    }

    async fn update_song(
        &mut self,
        path: &str,
        metadata: &Track,
        fingerprint: &str,
    ) -> Result<SqliteQueryResult, DbError> {
        sqlx::query!(
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
            self.timestamp,
            metadata.artist,
            metadata.title,
            metadata.album,
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
        .execute(&mut self.tran)
        .await
        .map_err(DbError::DbError)
    }
}
