CREATE TABLE IF NOT EXISTS deleted_song (
    deleted_song_id INTEGER PRIMARY KEY NOT NULL,
    song_id INTEGER NOT NULL,
    UNIQUE (song_id)
)