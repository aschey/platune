use super::Manager;
use crate::{config::MemoryConfig, database::Database};
use normpath::PathExt;
use pretty_assertions::assert_eq;
use std::{
    fs,
    path::{PathBuf, MAIN_SEPARATOR},
    sync::Arc,
};
use tempfile::{tempdir, TempDir};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_add_folders() {
    let (_, manager) = setup().await;

    let temp = tempdir().unwrap();
    let temp_str = temp.path().to_string_lossy().to_string();

    let test1 = temp.path().join("test1");
    let test2 = temp.path().join("test2");
    fs::create_dir_all(test1).unwrap();
    fs::create_dir_all(test2).unwrap();

    manager
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test1{MAIN_SEPARATOR}"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test1"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test2"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test2{MAIN_SEPARATOR}"))
        .await
        .unwrap();
    let folders = manager.get_all_folders().await.unwrap();

    assert_normalized(
        vec![
            format!(r"{temp_str}{MAIN_SEPARATOR}test1{MAIN_SEPARATOR}"),
            format!(r"{temp_str}{MAIN_SEPARATOR}test2{MAIN_SEPARATOR}"),
        ],
        folders,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_change_mount() {
    let (_, manager) = setup().await;

    let temp = tempdir().unwrap();
    let temp_str = temp.path().to_string_lossy().to_string();
    let test = temp.path().join("test");
    fs::create_dir_all(test).unwrap();

    manager.register_drive(&temp_str).await.unwrap();
    manager
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test{MAIN_SEPARATOR}"))
        .await
        .unwrap();
    let folders1 = manager.get_all_folders().await.unwrap();

    let temp2 = tempdir().unwrap();
    fs::create_dir_all(temp2.path().join("test")).unwrap();
    let temp_str2 = temp2.path().to_string_lossy().to_string();

    manager.register_drive(&temp_str2).await.unwrap();
    let folders2 = manager.get_all_folders().await.unwrap();

    assert_normalized(
        vec![format!(r"{temp_str}{MAIN_SEPARATOR}test{MAIN_SEPARATOR}")],
        folders1,
    );
    assert_normalized(
        vec![format!(r"{temp_str2}{MAIN_SEPARATOR}test{MAIN_SEPARATOR}")],
        folders2,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_change_mount_after() {
    let (_, manager) = setup().await;

    let temp = tempdir().unwrap();
    let temp_str = temp.path().to_string_lossy().to_string();
    let test = temp.path().join("test");
    fs::create_dir_all(test).unwrap();

    manager
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test{MAIN_SEPARATOR}"))
        .await
        .unwrap();
    manager.register_drive(&temp_str).await.unwrap();
    let folders1 = manager.get_all_folders().await.unwrap();

    let temp2 = tempdir().unwrap();
    fs::create_dir_all(temp2.path().join("test")).unwrap();
    let temp_str2 = temp2.path().to_string_lossy().to_string();

    manager.register_drive(&temp_str2).await.unwrap();
    let folders2 = manager.get_all_folders().await.unwrap();

    assert_normalized(
        vec![format!(r"{temp_str}{MAIN_SEPARATOR}test{MAIN_SEPARATOR}")],
        folders1,
    );
    assert_normalized(
        vec![format!(r"{temp_str2}{MAIN_SEPARATOR}test{MAIN_SEPARATOR}")],
        folders2,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_multiple_mounts() {
    let (db, manager) = setup().await;

    let config2 = Arc::new(MemoryConfig::new_boxed());
    let manager2 = Manager::new(&db, config2);

    let temp = tempdir().unwrap();
    let temp_str = temp.path().to_string_lossy().to_string();
    let test = temp.path().join("test");
    fs::create_dir_all(test).unwrap();

    manager
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test"))
        .await
        .unwrap();
    manager
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test{MAIN_SEPARATOR}"))
        .await
        .unwrap();
    manager.register_drive(&temp_str).await.unwrap();
    let folders1 = manager.get_all_folders().await.unwrap();

    let temp2 = tempdir().unwrap();
    let temp_str2 = temp2.path().to_string_lossy().to_string();
    fs::create_dir_all(temp2.path().join("test")).unwrap();

    manager2.register_drive(&temp_str2).await.unwrap();
    let folders2 = manager2.get_all_folders().await.unwrap();

    assert_normalized(
        vec![format!(r"{temp_str}{MAIN_SEPARATOR}test{MAIN_SEPARATOR}")],
        folders1,
    );
    assert_normalized(
        vec![format!(r"{temp_str2}{MAIN_SEPARATOR}test{MAIN_SEPARATOR}")],
        folders2,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_reset_drive_id_if_missing() {
    let temp = TempDir::new().unwrap();
    let sql_path = temp.path().join("platune.db");

    let db = Database::connect(sql_path, true).await.unwrap();
    db.sync_database().await.unwrap();
    let config = Arc::new(MemoryConfig::new_boxed());
    let manager = Manager::new(&db, config.clone());

    let test = temp.path().join("test");
    let temp_str = temp.path().to_string_lossy().to_string();
    fs::create_dir_all(test).unwrap();

    manager
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test"))
        .await
        .unwrap();
    manager.register_drive(&temp_str).await.unwrap();

    let tempdir2 = TempDir::new().unwrap();
    let sql_path2 = tempdir2.path().join("platune.db");
    let db2 = Database::connect(sql_path2, true).await.unwrap();
    db2.sync_database().await.unwrap();

    let manager2 = Manager::new(&db2, config);

    manager2
        .add_folder(&format!(r"{temp_str}{MAIN_SEPARATOR}test"))
        .await
        .unwrap();
    manager2.register_drive(&temp_str).await.unwrap();

    let folders = manager2.get_all_folders().await.unwrap();

    assert_normalized(
        vec![format!(r"{temp_str}{MAIN_SEPARATOR}test{MAIN_SEPARATOR}")],
        folders,
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_validate_path() {
    let (_, manager) = setup().await;

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

fn normalize(s: &String) -> String {
    PathBuf::from(s)
        .normalize()
        .unwrap()
        .into_os_string()
        .to_string_lossy()
        .to_string()
}

fn normalize_all(strs: Vec<String>) -> Vec<String> {
    strs.iter().map(normalize).collect()
}

fn assert_normalized(left: Vec<String>, right: Vec<String>) {
    assert_eq!(normalize_all(left), normalize_all(right));
}
