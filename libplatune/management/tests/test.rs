use directories::BaseDirs;
use libplatune_management::{config::Config, database::Database};
use log::LevelFilter;
use sqlx::{
    pool::PoolConnection,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions, Sqlite, SqlitePool,
};
use std::{
    env,
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};
use tempfile::{tempdir, TempDir};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_add_folders() {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;

    config.add_folder("test1").await;
    config.add_folder("test1").await;
    config.add_folder("test2").await;
    let folders = config.get_all_folders().await;
    db.close().await;
    assert_eq!(vec!["test1", "test2"], folders);
}

async fn setup(tempdir: &TempDir) -> (Database, Config) {
    let sql_path = tempdir.path().join("platune.db");
    let config_path = tempdir.path().join("platuneconfig");
    let db = Database::connect(sql_path, true).await;
    db.migrate().await;
    let config = Config::new_from_path(&db, config_path);
    (db, config)
}
