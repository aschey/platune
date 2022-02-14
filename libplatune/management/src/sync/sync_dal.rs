use std::time::SystemTime;

use itertools::Itertools;
use katatsuki::ReadOnlyTrack;
use sqlx::{sqlite::SqliteQueryResult, Pool, Sqlite, Transaction};

use crate::{consts::MIN_LEN, db_error::DbError, spellfix::load_spellfix};

pub(crate) struct SyncDAL<'a> {
    tran: Transaction<'a, Sqlite>,
    timestamp: u32,
}

impl<'a> SyncDAL<'a> {
    pub(crate) async fn try_new(write_pool: Pool<Sqlite>) -> Result<SyncDAL<'a>, DbError> {
        let mut tran = write_pool
            .begin()
            .await
            .map_err(|e| DbError::DbError(format!("{e:?}")))?;
        load_spellfix(&mut tran).await?;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        Ok(Self { tran, timestamp })
    }

    pub(crate) async fn add_artist(&mut self, artist: &str) -> Result<SqliteQueryResult, DbError> {
        sqlx::query!(
            "INSERT OR IGNORE INTO artist(artist_name, created_date) values(?, ?);",
            artist,
            self.timestamp
        )
        .execute(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))
    }

    pub(crate) async fn add_album_artist(
        &mut self,
        album_artist: &str,
    ) -> Result<SqliteQueryResult, DbError> {
        sqlx::query!(
            "INSERT OR IGNORE INTO album_artist(album_artist_name, created_date) values(?, ?);",
            album_artist,
            self.timestamp
        )
        .execute(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))
    }

    pub(crate) async fn add_album(
        &mut self,
        album: &str,
        album_artist: &str,
    ) -> Result<SqliteQueryResult, DbError> {
        sqlx::query!(
            "
        INSERT OR IGNORE INTO album(album_name, album_artist_id, created_date) 
        values(?, (SELECT album_artist_id FROM album_artist WHERE album_artist_name = ?), ?);",
            album,
            album_artist,
            self.timestamp
        )
        .execute(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))
    }

    pub(crate) async fn sync_song(
        &mut self,
        path: &str,
        metadata: &ReadOnlyTrack,
        file_size: i64,
        fingerprint: &str,
    ) -> Result<SqliteQueryResult, DbError> {
        self.add_song(path, metadata, file_size, fingerprint)
            .await?;
        self.update_song(path, metadata, file_size, fingerprint)
            .await
    }

    pub(crate) async fn update_missing_songs(&mut self, path: String) -> Result<(), DbError> {
        // Add songs not found in the last scan attempt to the list of deleted songs
        let path = path + "%";
        sqlx::query!(
            "
            INSERT INTO deleted_song(song_id)
            SELECT song_id FROM song WHERE last_scanned_date < ?
            AND song_path like ?
            ON CONFLICT DO NOTHING;
            ",
            self.timestamp,
            path
        )
        .execute(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))?;

        // If a song was previously missing but was found in the most recent scan,
        // remove it from the list of deleted songs
        sqlx::query!(
            "
            DELETE FROM deleted_song as ds
            WHERE EXISTS(SELECT 1 FROM song s WHERE s.song_id = ds.song_id AND s.last_scanned_date = ?)
            ",
            self.timestamp
        )
        .execute(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))?;

        Ok(())
    }

    pub(crate) async fn sync_spellfix(&mut self) -> Result<(), DbError> {
        sqlx::query(
            "
            INSERT INTO search_spellfix(word)
            SELECT term
            FROM search_vocab
            WHERE term not in (
                SELECT word
                FROM search_spellfix
            );
            ",
        )
        .execute(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))?;

        sqlx::query(
            "
            DELETE FROM search_spellfix
            WHERE word NOT IN (
                SELECT term
                FROM search_vocab
            );
            ",
        )
        .execute(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))?;

        Ok(())
    }

    pub(crate) async fn get_long_entries(&mut self) -> Result<Vec<String>, DbError> {
        let long_vals = sqlx::query!(
            r#"
            SELECT entry_value as "entry_value: String"
            FROM search_index
            WHERE length(entry_value) >= $1
            and entry_type != 'song';
            "#,
            MIN_LEN as i32
        )
        .fetch_all(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))?
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
            INSERT INTO search_spellfix(word, soundslike)
            SELECT $1, $2
            WHERE NOT EXISTS (
                SELECT 1 FROM search_spellfix WHERE word = $1
            );
        ",
        )
        .bind(entry_value)
        .bind(acronym)
        .execute(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))?;

        if acronym.contains('&') {
            let replaced = acronym.replace("&", "a");
            self.insert_alt_alias(entry_value, &replaced).await?;
        }

        Ok(())
    }

    pub(crate) async fn commit(self) -> Result<(), DbError> {
        self.tran
            .commit()
            .await
            .map_err(|e| DbError::DbError(format!("{e:?}")))
    }

    async fn insert_alt_alias(
        &mut self,
        entry_value: &str,
        acronym: &str,
    ) -> Result<SqliteQueryResult, DbError> {
        sqlx::query(
            "
            INSERT INTO search_spellfix(word, soundslike)
            SELECT $1, $2
            WHERE  (
                SELECT count(1) FROM search_spellfix WHERE word = $1
            ) < 2;
        ",
        )
        .bind(entry_value)
        .bind(acronym)
        .execute(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))
    }

    async fn add_song(
        &mut self,
        path: &str,
        metadata: &ReadOnlyTrack,
        file_size: i64,
        fingerprint: &str,
    ) -> Result<SqliteQueryResult, DbError> {
        sqlx::query!(
            "
        INSERT INTO song(
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
            file_size,
            album_art_path,
            fingerprint
            )
            values
            (
                ?, ?, ?, ?,
                (SELECT artist_id FROM artist WHERE artist_name = ?), 
                ?, 
                (
                    SELECT album_id FROM album a
                    INNER JOIN album_artist aa ON a.album_artist_id = aa.album_artist_id
                    WHERE a.album_name = ? AND aa.album_artist_name = ?
                ), 
                ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
            )
            ON CONFLICT(song_path) DO UPDATE
            SET last_scanned_date = ?;
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
            file_size,
            "",
            fingerprint,
            self.timestamp
        )
        .execute(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))
    }

    async fn update_song(
        &mut self,
        path: &str,
        metadata: &ReadOnlyTrack,
        file_size: i64,
        fingerprint: &str,
    ) -> Result<SqliteQueryResult, DbError> {
        sqlx::query!(
            "
        UPDATE song
            SET modified_date = $2,
            artist_id = (SELECT artist_id FROM artist WHERE artist_name = $3),
            song_title = $4,
            album_id = (SELECT album_id FROM album WHERE album_name = $5),
            track_number = $6,
            disc_number = $7,
            song_year = $8,
            song_month = $9,
            song_day = $10,
            duration = $11,
            sample_rate = $12,
            bit_rate = $13,
            file_size = $14,
            album_art_path = $15,
            fingerprint = $16
        WHERE song_path = $1 AND fingerprint != $16;
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
            file_size,
            "",
            fingerprint
        )
        .execute(&mut self.tran)
        .await
        .map_err(|e| DbError::DbError(format!("{e:?}")))
    }
}
