CREATE TABLE IF NOT EXISTS mount (
    mount_id INTEGER PRIMARY KEY NOT NULL,
    unix_path TEXT NOT NULL,
    windows_path TEXT NOT NULL
)