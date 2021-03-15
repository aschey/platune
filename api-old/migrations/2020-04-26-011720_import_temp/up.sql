CREATE TABLE IF NOT EXISTS import_temp (
    import_id INTEGER PRIMARY KEY NOT NULL,
    import_song_path_windows TEXT NOT NULL,
    import_song_path_unix TEXT NOT NULL,
    import_artist TEXT NOT NULL,
    import_album_artist TEXT NOT NULL,
    import_title TEXT NOT NULL,
    import_album TEXT NOT NULL,
    import_track_number INTEGER NOT NULL,
    import_disc_number INTEGER NOT NULL,
    import_year INTEGER NOT NULL,
    import_duration INTEGER NOT NULL,
    import_sample_rate INTEGER NOT NULL,
    import_bit_rate INTEGER NOT NULL,
    import_album_art BLOB NULL,
    UNIQUE(import_song_path_unix, import_song_path_windows)
)