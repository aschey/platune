CREATE TABLE IF NOT EXISTS import_temp (
    import_id INTEGER PRIMARY KEY NOT NULL,
    import_song_path TEXT NOT NULL UNIQUE,
    import_modified_date INTEGER NOT NULL,
    import_artist TEXT NOT NULL,
    import_title TEXT NOT NULL,
    import_album TEXT NOT NULL
)