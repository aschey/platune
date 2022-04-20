use super::Manager;
use crate::{config::MemoryConfig, database::Database};
use pretty_assertions::assert_eq;
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_add_folders() {
    let (_, mut config) = setup().await;
    config.delim = r"\";

    config.add_folder(r"test1\").await.unwrap();
    config.add_folder("test1").await.unwrap();
    config.add_folder("test2").await.unwrap();
    config.add_folder(r"test2\\").await.unwrap();
    let folders = config.get_all_folders().await.unwrap();

    assert_eq!(vec![r"test1\", r"test2\"], folders);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_change_mount() {
    let (_, mut manager) = setup().await;
    manager.delim = r"\";
    manager.validate_paths = false;

    manager.register_drive(r"C:\\").await.unwrap();
    manager.add_folder(r"C:\test").await.unwrap();
    manager.add_folder(r"C:\\test\\").await.unwrap();
    let folders1 = manager.get_all_folders().await.unwrap();
    manager.register_drive(r"D:\").await.unwrap();
    let folders2 = manager.get_all_folders().await.unwrap();

    assert_eq!(vec![r"C:\test\"], folders1);
    assert_eq!(vec![r"D:\test\"], folders2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_change_mount_after() {
    let (_, mut manager) = setup().await;
    manager.delim = r"\";
    manager.validate_paths = false;

    manager.add_folder(r"C:\test").await.unwrap();
    manager.add_folder(r"C:\\test\\").await.unwrap();
    manager.register_drive(r"C:\").await.unwrap();
    let folders1 = manager.get_all_folders().await.unwrap();
    manager.register_drive(r"D:\").await.unwrap();
    let folders2 = manager.get_all_folders().await.unwrap();

    assert_eq!(vec![r"C:\test\"], folders1);
    assert_eq!(vec![r"D:\test\"], folders2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_multiple_mounts() {
    let (db, mut manager) = setup().await;

    let config2 = Arc::new(MemoryConfig::new_boxed());
    let mut manager2 = Manager::new(&db, config2);
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

    assert_eq!(vec![r"C:\test\"], folders1);
    assert_eq!(vec![r"D:\test\"], folders2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_reset_drive_id_if_missing() {
    let tempdir = TempDir::new().unwrap();
    let sql_path = tempdir.path().join("platune.db");

    let db = Database::connect(sql_path, true).await.unwrap();
    db.migrate().await.unwrap();
    let config = Arc::new(MemoryConfig::new_boxed());
    let mut manager = Manager::new(&db, config.clone());
    manager.delim = r"\";
    manager.validate_paths = false;

    manager.add_folder(r"C:\test").await.unwrap();
    manager.register_drive(r"C:\").await.unwrap();

    let tempdir2 = TempDir::new().unwrap();
    let sql_path2 = tempdir2.path().join("platune.db");
    let db2 = Database::connect(sql_path2, true).await.unwrap();
    db2.migrate().await.unwrap();

    let mut manager2 = Manager::new(&db2, config);
    manager2.delim = r"\";
    manager2.validate_paths = false;

    manager2.add_folder(r"C:\test").await.unwrap();
    manager2.register_drive(r"C:\").await.unwrap();

    let folders = manager2.get_all_folders().await.unwrap();

    assert_eq!(vec![r"C:\test\"], folders);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_validate_path() {
    let (_, mut manager) = setup().await;

    manager.delim = r"\";

    let res = manager.register_drive(r"/some/invalid/path").await;

    assert!(res.is_err());
}

async fn setup() -> (Database, Manager) {
    let db = Database::connect_in_memory().await.unwrap();
    db.migrate().await.unwrap();
    let config = Arc::new(MemoryConfig::new_boxed());
    let manager = Manager::new(&db, config);
    (db, manager)
}
