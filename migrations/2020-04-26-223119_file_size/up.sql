CREATE TABLE IF NOT EXISTS file_size (
    file_size_id INTEGER PRIMARY KEY NOT NULL,
    song_id INTEGER NOT NULL UNIQUE,
    song_file_size INTEGER NOT NULL,
    file_modified_date INTEGER NOT NULL,
    FOREIGN KEY(song_id) REFERENCES song(song_id)
)