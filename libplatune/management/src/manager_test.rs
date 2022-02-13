use super::Manager;
use crate::{config::Config, database::Database};
use pretty_assertions::assert_eq;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::timeout;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_add_folders() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut config) = setup(&tempdir).await;
    config.delim = r"\";

    config.add_folder(r"test1\").await.unwrap();
    config.add_folder("test1").await.unwrap();
    config.add_folder("test2").await.unwrap();
    config.add_folder(r"test2\\").await.unwrap();
    let folders = config.get_all_folders().await.unwrap();

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(vec![r"test1\", r"test2\"], folders);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_change_mount() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    manager.delim = r"\";
    manager.validate_paths = false;

    manager.register_drive(r"C:\\").await.unwrap();
    manager.add_folder(r"C:\test").await.unwrap();
    manager.add_folder(r"C:\\test\\").await.unwrap();
    let folders1 = manager.get_all_folders().await.unwrap();
    manager.register_drive(r"D:\").await.unwrap();
    let folders2 = manager.get_all_folders().await.unwrap();

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(vec![r"C:\test\"], folders1);
    assert_eq!(vec![r"D:\test\"], folders2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_change_mount_after() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    manager.delim = r"\";
    manager.validate_paths = false;

    manager.add_folder(r"C:\test").await.unwrap();
    manager.add_folder(r"C:\\test\\").await.unwrap();
    manager.register_drive(r"C:\").await.unwrap();
    let folders1 = manager.get_all_folders().await.unwrap();
    manager.register_drive(r"D:\").await.unwrap();
    let folders2 = manager.get_all_folders().await.unwrap();

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(vec![r"C:\test\"], folders1);
    assert_eq!(vec![r"D:\test\"], folders2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_multiple_mounts() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let config_path2 = tempdir.path().join("platuneconfig2");
    let config2 = Config::new_from_path(config_path2).unwrap();
    let mut manager2 = Manager::new(&db, &config2);
    manager.delim = r"\";
    manager.validate_paths = false;
    manager2.delim = r"\";
    manager2.validate_paths = false;

    manager.add_folder(r"C:\test").await.unwrap();
    manager.add_folder(r"C:\\test\\").await.unwrap();
    manager.register_drive(r"C:\").await.unwrap();
    let folders1 = manager.get_all_folders().await.unwrap();
    manager2.register_drive(r"D:\").await.unwrap();
    let folders2 = manager2.get_all_folders().await.unwrap();

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(vec![r"C:\test\"], folders1);
    assert_eq!(vec![r"D:\test\"], folders2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_reset_drive_id_if_missing() {
    let tempdir = TempDir::new().unwrap();
    let sql_path = tempdir.path().join("platune.db");
    let config_path = tempdir.path().join("platuneconfig");
    let db = Database::connect(sql_path, true).await.unwrap();
    db.migrate().await.unwrap();
    let config = Config::new_from_path(config_path.clone()).unwrap();
    let mut manager = Manager::new(&db, &config);
    manager.delim = r"\";
    manager.validate_paths = false;

    manager.add_folder(r"C:\test").await.unwrap();
    manager.register_drive(r"C:\").await.unwrap();

    let tempdir2 = TempDir::new().unwrap();
    let sql_path2 = tempdir2.path().join("platune.db");
    let db2 = Database::connect(sql_path2, true).await.unwrap();
    db2.migrate().await.unwrap();

    let mut manager2 = Manager::new(&db2, &config);
    manager2.delim = r"\";
    manager2.validate_paths = false;

    manager2.add_folder(r"C:\test").await.unwrap();
    manager2.register_drive(r"C:\").await.unwrap();

    let folders = manager2.get_all_folders().await.unwrap();

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();
    timeout(Duration::from_secs(5), db2.close())
        .await
        .unwrap_or_default();

    assert_eq!(vec![r"C:\test\"], folders);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_validate_path() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;

    manager.delim = r"\";

    let res = manager.register_drive(r"/some/invalid/path").await;

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert!(res.is_err());
}

async fn setup(tempdir: &TempDir) -> (Database, Manager) {
    let sql_path = tempdir.path().join("platune.db");
    let config_path = tempdir.path().join("platuneconfig");
    let db = Database::connect(sql_path, true).await.unwrap();
    db.migrate().await.unwrap();
    let config = Config::new_from_path(config_path).unwrap();
    let manager = Manager::new(&db, &config);
    (db, manager)
}
