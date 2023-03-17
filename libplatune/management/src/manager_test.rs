use super::Manager;
use crate::{config::MemoryConfig, database::Database};
use pretty_assertions::assert_eq;
use std::{fs, sync::Arc};
use tempfile::{tempdir, TempDir};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_add_folders() {
    let (_, mut manager) = setup().await;
    manager.delim = r"\";

    let temp = tempdir().unwrap();
    let temp_str = temp.path().to_string_lossy().to_string().replace('/', r"\");

    let test1 = temp.path().join("test1");
    let test2 = temp.path().join("test2");
    fs::create_dir_all(test1).unwrap();
    fs::create_dir_all(test2).unwrap();

    manager
        .add_folder(&format!(r"{temp_str}\test1\"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}\test1"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}\test2"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}\test2\"))
        .await
        .unwrap();
    let folders = manager.get_all_folders().await.unwrap();

    assert_eq!(
        vec![format!(r"{temp_str}\test1\"), format!(r"{temp_str}\test2\")],
        folders
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_change_mount() {
    let (_, mut manager) = setup().await;
    manager.delim = r"\";

    let temp = tempdir().unwrap();
    let temp_str = temp.path().to_string_lossy().to_string().replace('/', r"\");
    let test = temp.path().join("test");
    fs::create_dir_all(test).unwrap();

    manager.register_drive(&temp_str).await.unwrap();
    manager
        .add_folder(&format!(r"{temp_str}\test"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}\test\"))
        .await
        .unwrap();
    let folders1 = manager.get_all_folders().await.unwrap();

    let temp2 = tempdir().unwrap();
    let temp_str2 = temp2
        .path()
        .to_string_lossy()
        .to_string()
        .replace('/', r"\");

    manager.register_drive(&temp_str2).await.unwrap();
    let folders2 = manager.get_all_folders().await.unwrap();

    assert_eq!(vec![format!(r"{temp_str}\test\")], folders1);
    assert_eq!(vec![format!(r"{temp_str2}\test\")], folders2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_change_mount_after() {
    let (_, mut manager) = setup().await;
    manager.delim = r"\";

    let temp = tempdir().unwrap();
    let temp_str = temp.path().to_string_lossy().to_string().replace('/', r"\");
    let test = temp.path().join("test");
    fs::create_dir_all(test).unwrap();

    manager
        .add_folder(&format!(r"{temp_str}\test"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}\test\"))
        .await
        .unwrap();
    manager.register_drive(&temp_str).await.unwrap();
    let folders1 = manager.get_all_folders().await.unwrap();

    let temp2 = tempdir().unwrap();
    let temp_str2 = temp2
        .path()
        .to_string_lossy()
        .to_string()
        .replace('/', r"\");

    manager.register_drive(&temp_str2).await.unwrap();
    let folders2 = manager.get_all_folders().await.unwrap();

    assert_eq!(vec![format!(r"{temp_str}\test\")], folders1);
    assert_eq!(vec![format!(r"{temp_str2}\test\")], folders2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_multiple_mounts() {
    let (db, mut manager) = setup().await;

    let config2 = Arc::new(MemoryConfig::new_boxed());
    let mut manager2 = Manager::new(&db, config2);
    manager.delim = r"\";
    manager2.delim = r"\";

    let temp = tempdir().unwrap();
    let temp_str = temp.path().to_string_lossy().to_string().replace('/', r"\");
    let test = temp.path().join("test");
    fs::create_dir_all(test).unwrap();

    manager
        .add_folder(&format!(r"{temp_str}\test"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}\test\"))
        .await
        .unwrap();
    manager.register_drive(&temp_str).await.unwrap();
    let folders1 = manager.get_all_folders().await.unwrap();

    let temp2 = tempdir().unwrap();
    let temp_str2 = temp2
        .path()
        .to_string_lossy()
        .to_string()
        .replace('/', r"\");
    fs::create_dir_all(temp2.path().join("test")).unwrap();

    manager2.register_drive(&temp_str2).await.unwrap();
    let folders2 = manager2.get_all_folders().await.unwrap();

    assert_eq!(vec![format!(r"{temp_str}\test\")], folders1);
    assert_eq!(vec![format!(r"{temp_str2}\test\")], folders2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_reset_drive_id_if_missing() {
    let temp = TempDir::new().unwrap();
    let sql_path = temp.path().join("platune.db");

    let db = Database::connect(sql_path, true).await.unwrap();
    db.sync_database().await.unwrap();
    let config = Arc::new(MemoryConfig::new_boxed());
    let mut manager = Manager::new(&db, config.clone());
    manager.delim = r"\";

    let test = temp.path().join("test");
    let temp_str = temp.path().to_string_lossy().to_string().replace('/', r"\");
    fs::create_dir_all(test).unwrap();

    manager
        .add_folder(&format!(r"{temp_str}\test"))
        .await
        .unwrap();
    manager.register_drive(&temp_str).await.unwrap();

    let tempdir2 = TempDir::new().unwrap();
    let sql_path2 = tempdir2.path().join("platune.db");
    let db2 = Database::connect(sql_path2, true).await.unwrap();
    db2.sync_database().await.unwrap();

    let mut manager2 = Manager::new(&db2, config);
    manager2.delim = r"\";

    manager2
        .add_folder(&format!(r"{temp_str}\test"))
        .await
        .unwrap();
    manager2.register_drive(&temp_str).await.unwrap();

    let folders = manager2.get_all_folders().await.unwrap();

    assert_eq!(vec![format!(r"{temp_str}\test\")], folders);
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
    db.sync_database().await.unwrap();
    let config = Arc::new(MemoryConfig::new_boxed());
    let manager = Manager::new(&db, config);
    (db, manager)
}
