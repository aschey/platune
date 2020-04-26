CREATE TABLE IF NOT EXISTS album (
    album_id INTEGER PRIMARY KEY NOT NULL,
    album_name TEXT NOT NULL,
    is_compilation BOOLEAN NOT NULL,
    release_date INTEGER NOT NULL,
    album_artist_id INTEGER NOT NULL,
    disc_number INTEGER NOT NULL,
    FOREIGN KEY(album_artist_id) REFERENCES artist(artist_id)
)