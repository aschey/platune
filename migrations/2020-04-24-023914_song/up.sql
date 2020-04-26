CREATE TABLE IF NOT EXISTS song (
    song_id INTEGER PRIMARY KEY NOT NULL,
    song_path_unix TEXT NOT NULL UNIQUE,
    song_path_windows TEXT NOT NULL UNIQUE,
    modified_date INTEGER NOT NULL,
    artist_id INTEGER NOT NULL,
    song_title TEXT NOT NULL,
    album_id INTEGER NOT NULL,
    track_number INTEGER NOT NULL,
    play_count INTEGER NOT NULL,
    file_size INTEGER NOT NULL,
    disc_number INTEGER NOT NULL,
    song_year INTEGER NOT NULL,
    is_deleted BOOLEAN NOT NULL,
    FOREIGN KEY(artist_id) REFERENCES artist(artist_id),
    FOREIGN KEY(album_id) REFERENCES album(album_id)
)