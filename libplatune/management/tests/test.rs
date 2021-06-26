
use libplatune_management::{config::Config, database::Database};

use sqlx::{
    pool::PoolConnection,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions, Sqlite, SqlitePool,
};

use tempfile::{TempDir};

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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_change_mount() {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;

    config.register_drive("C:\\").await;
    config.add_folder("C:\\test").await;
    let folders1 = config.get_all_folders().await;
    config.register_drive("D:\\").await;
    let folders2 = config.get_all_folders().await;
    db.close().await;
    assert_eq!(vec!["C:\\test"], folders1);
    assert_eq!(vec!["D:\\test"], folders2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_change_mount_after() {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;

    config.add_folder("C:\\test").await;
    config.register_drive("C:\\").await;
    let folders1 = config.get_all_folders().await;
    config.register_drive("D:\\").await;
    let folders2 = config.get_all_folders().await;
    db.close().await;
    assert_eq!(vec!["C:\\test"], folders1);
    assert_eq!(vec!["D:\\test"], folders2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_multiple_mounts() {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;
    let config_path2 = tempdir.path().join("platuneconfig2");
    let config2 = Config::new_from_path(&db, config_path2);

    config.add_folder("C:\\test").await;
    config.register_drive("C:\\").await;
    let folders1 = config.get_all_folders().await;
    config2.register_drive("D:\\").await;
    let folders2 = config2.get_all_folders().await;
    db.close().await;
    assert_eq!(vec!["C:\\test"], folders1);
    assert_eq!(vec!["D:\\test"], folders2);
}

async fn setup(tempdir: &TempDir) -> (Database, Config) {
    let sql_path = tempdir.path().join("platune.db");
    let config_path = tempdir.path().join("platuneconfig");
    let db = Database::connect(sql_path, true).await;
    db.migrate().await;
    let config = Config::new_from_path(&db, config_path);
    (db, config)
}
