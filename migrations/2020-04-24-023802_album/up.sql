CREATE TABLE IF NOT EXISTS album (
    album_id INTEGER PRIMARY KEY NOT NULL,
    album_name TEXT NOT NULL,
    album_year INTEGER NOT NULL,
    album_month INTEGER NOT NULL,
    album_day INTEGER NOT NULL,
    album_artist_id INTEGER NOT NULL,
    UNIQUE(album_name, album_artist_id)
    FOREIGN KEY(album_artist_id) REFERENCES album_artist(album_artist_id)
)