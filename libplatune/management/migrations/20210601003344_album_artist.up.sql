CREATE TABLE IF NOT EXISTS album_artist (
    album_artist_id INTEGER PRIMARY KEY NOT NULL,
    album_artist_name TEXT NOT NULL COLLATE NOCASE,
    created_date INTEGER NOT NULL,
    UNIQUE (album_artist_name COLLATE NOCASE)
)