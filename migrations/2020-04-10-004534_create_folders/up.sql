CREATE TABLE IF NOT EXISTS folder (
    folder_id INTEGER PRIMARY KEY NOT NULL,
    full_path_unix TEXT NOT NULL,
    full_path_windows TEXT NOT NULL
)