CREATE TABLE IF NOT EXISTS album (
    album_id INTEGER PRIMARY KEY NOT NULL,
    album_name TEXT NOT NULL COLLATE NOCASE,
    artist_id INTEGER NOT NULL,
    created_date INTEGER NOT NULL,
    UNIQUE(album_name, artist_id) FOREIGN KEY(artist_id) REFERENCES artist(artist_id)
)