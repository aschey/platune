use std::fs::{self, create_dir, create_dir_all};

use libplatune_management::{config::Config, database::Database};
use tempfile::TempDir;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_sync_empty() {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    create_dir(music_dir.clone()).unwrap();
    config.add_folder(music_dir.to_str().unwrap()).await;
    let mut receiver = config.sync().await;
    let mut msgs = vec![];
    while let Some(msg) = receiver.recv().await {
        msgs.push(msg);
    }
    db.close().await;
    assert_eq!(vec![1.], msgs);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
pub async fn test_sync_basic() {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    fs::copy(
        "../player/tests/assets/test.mp3",
        inner_dir.join("test.mp3"),
    )
    .unwrap();
    fs::copy(
        "../player/tests/assets/test2.mp3",
        inner_dir.join("test2.mp3"),
    )
    .unwrap();
    fs::copy(
        "../player/tests/assets/test3.mp3",
        inner_dir.join("test3.mp3"),
    )
    .unwrap();

    config.add_folder(music_dir.to_str().unwrap()).await;
    let mut receiver = config.sync().await;

    let mut msgs = vec![];
    while let Some(msg) = receiver.recv().await {
        msgs.push(msg);
    }

    db.close().await;
    assert_eq!(vec![0., 1.], msgs);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test_search() {
    let tempdir = TempDir::new().unwrap();
    let (db, config) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    fs::copy(
        "../player/tests/assets/test.mp3",
        inner_dir.join("test.mp3"),
    )
    .unwrap();
    fs::copy(
        "../player/tests/assets/test2.mp3",
        inner_dir.join("test2.mp3"),
    )
    .unwrap();
    fs::copy(
        "../player/tests/assets/test3.mp3",
        inner_dir.join("test3.mp3"),
    )
    .unwrap();

    let t = katatsuki::Track::from_path(&inner_dir.join("test.mp3"), None).unwrap();

    t.set_title("asdf");
    t.save();
    drop(t);

    config.add_folder(music_dir.to_str().unwrap()).await;
    let mut receiver = config.sync().await;

    while let Some(_) = receiver.recv().await {}

    let res = db.search("asdf", Default::default()).await;
    db.close().await;
    assert!(res.len() > 0);
}

async fn setup(tempdir: &TempDir) -> (Database, Config) {
    let sql_path = tempdir.path().join("platune.db");
    let config_path = tempdir.path().join("platuneconfig");
    let db = Database::connect(sql_path, true).await;
    db.migrate().await;
    let config = Config::new_from_path(&db, config_path);
    (db, config)
}
