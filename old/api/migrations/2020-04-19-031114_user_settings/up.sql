CREATE TABLE IF NOT EXISTS user_setting (
    setting_id INTEGER PRIMARY KEY NOT NULL,
    setting_name TEXT NOT NULL UNIQUE,
    setting_value TEXT NOT NULL UNIQUE
)