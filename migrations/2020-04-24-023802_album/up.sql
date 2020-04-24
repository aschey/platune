CREATE TABLE IF NOT EXISTS album (
    album_id INTEGER PRIMARY KEY NOT NULL,
    album_name TEXT NOT NULL,
    is_compilation BOOLEAN NOT NULL,
    release_date INT NOT NULL
)