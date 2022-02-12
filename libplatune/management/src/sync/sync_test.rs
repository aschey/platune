use crate::{config::Config, database::Database, manager::Manager};
use futures::StreamExt;
use itertools::Itertools;
use rstest::*;
use std::{
    fs::{self, create_dir, create_dir_all},
    path::Path,
    time::Duration,
};
use tempfile::TempDir;
use tokio::time::timeout;
use tracing::{info, Level};

#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt()
        .pretty()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_test_writer()
        .with_max_level(Level::INFO)
        .try_init()
        .unwrap_or_default();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_empty() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    create_dir(music_dir.clone()).unwrap();
    manager
        .add_folder(music_dir.to_str().unwrap())
        .await
        .unwrap();
    let mut receiver = manager.sync(None).await.unwrap();
    let mut msgs = vec![];
    while let Some(msg) = receiver.next().await {
        msgs.push(msg.unwrap());
    }

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(vec![0., 1.], msgs);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_no_folder() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;

    let mut receiver = manager.sync(None).await.unwrap();
    let mut msgs = vec![];
    while let Some(msg) = receiver.next().await {
        msgs.push(msg.unwrap());
    }

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(Vec::<f32>::new(), msgs);
}

#[rstest(use_mount, case(true), case(false))]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_basic(use_mount: bool) {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    let paths = vec![
        inner_dir.join("test.mp3"),
        inner_dir.join("test2.mp3"),
        inner_dir.join("test3.mp3"),
    ];
    fs::copy("../test_assets/test.mp3", &paths[0]).unwrap();
    fs::copy("../test_assets/test2.mp3", &paths[1]).unwrap();
    fs::copy("../test_assets/test3.mp3", &paths[2]).unwrap();

    manager
        .add_folder(music_dir.to_str().unwrap())
        .await
        .unwrap();

    if use_mount {
        let mount = music_dir.parent().unwrap();
        manager.register_drive(mount).await.unwrap();
    }
    let mut receiver = manager.sync(None).await.unwrap();

    let mut msgs = vec![];
    while let Some(msg) = receiver.next().await {
        msgs.push(msg.unwrap());
    }

    let msgs = msgs.into_iter().skip_while(|m| *m == 0.0).collect_vec();
    assert_eq!(vec![0.2, 0.4, 0.6, 0.8, 1.0], msgs);

    for path in paths {
        assert!(manager.get_song_by_path(path).await.unwrap().is_some());
    }

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_multiple() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");

    let mut paths = vec![];
    for i in 0..10 {
        create_dir_all(inner_dir.clone().join(format!("test{i}"))).unwrap();

        let dir1 = inner_dir.join(format!("test{i}/test.mp3"));
        fs::copy("../test_assets/test.mp3", &dir1).unwrap();
        paths.push(dir1);

        let dir2 = inner_dir.join(format!("test{i}/test2.mp3"));
        fs::copy("../test_assets/test2.mp3", &dir2).unwrap();
        paths.push(dir2);

        let dir3 = inner_dir.join(format!("test{i}/test3.mp3"));
        fs::copy("../test_assets/test3.mp3", &dir3).unwrap();
        paths.push(dir3);
    }

    manager
        .add_folder(music_dir.to_str().unwrap())
        .await
        .unwrap();

    let mut receiver1 = manager.sync(None).await.unwrap();
    let mut receiver2 = manager.sync(None).await.unwrap();

    tokio::spawn(async move { while receiver1.next().await.is_some() {} })
        .await
        .unwrap();

    tokio::spawn(async move { while receiver2.next().await.is_some() {} })
        .await
        .unwrap();

    for path in paths {
        assert!(manager.get_song_by_path(path).await.unwrap().is_some());
    }

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();
}

async fn setup_delete(inner_dir: &Path, music_dir: &Path, manager: &mut Manager) {
    fs::copy("../test_assets/test.mp3", inner_dir.join("test.mp3")).unwrap();
    fs::copy("../test_assets/test2.mp3", inner_dir.join("test2.mp3")).unwrap();
    let last_song = inner_dir.join("test3.mp3");
    fs::copy("../test_assets/test3.mp3", last_song.clone()).unwrap();

    manager
        .add_folder(music_dir.to_str().unwrap())
        .await
        .unwrap();
    let mut receiver = manager.sync(None).await.unwrap();

    while receiver.next().await.is_some() {}

    fs::remove_file(last_song.clone()).unwrap();
    // We store unix timestamps at a granularity of seconds so we need to wait for enough time to pass
    std::thread::sleep(Duration::from_secs(2));

    let mut receiver = manager.sync(None).await.unwrap();
    while receiver.next().await.is_some() {}

    // sync twice after deleting to ensure no unique constraint errors
    let mut receiver = manager.sync(None).await.unwrap();
    while receiver.next().await.is_some() {}
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_delete() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    setup_delete(&inner_dir, &music_dir, &mut manager).await;

    let deleted = manager.get_deleted_songs().await.unwrap();

    manager
        .delete_tracks(vec![deleted[0].song_id])
        .await
        .unwrap();

    let deleted2 = manager.get_deleted_songs().await.unwrap();

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    let last_song = inner_dir.join("test3.mp3");

    assert_eq!(1, deleted.len());
    assert_eq!(
        deleted[0].song_path,
        last_song.to_string_lossy().to_string().replace("\\", "/")
    );

    // TODO: add test to check if number of songs decreased once we have an endpoint to get all songs
    assert_eq!(0, deleted2.len());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_delete_and_readd() {
    let tempdir = TempDir::new().unwrap();
    let (db, mut manager) = setup(&tempdir).await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    setup_delete(&inner_dir, &music_dir, &mut manager).await;
    let deleted = manager.get_deleted_songs().await.unwrap();

    let last_song = inner_dir.join("test3.mp3");

    fs::copy("../test_assets/test3.mp3", last_song.clone()).unwrap();

    let mut receiver = manager.sync(None).await.unwrap();
    while receiver.next().await.is_some() {}

    let deleted2 = manager.get_deleted_songs().await.unwrap();

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();

    assert_eq!(1, deleted.len());
    assert_eq!(
        deleted[0].song_path,
        last_song.to_string_lossy().to_string().replace("\\", "/")
    );

    // TODO: add test to check if number of songs decreased once we have an endpoint to get all songs
    assert_eq!(0, deleted2.len());
}

async fn setup(tempdir: &TempDir) -> (Database, Manager) {
    let sql_path = tempdir.path().join("platune.db");
    info!("{:?}", sql_path);
    let config_path = tempdir.path().join("platuneconfig");
    let db = Database::connect(sql_path, true).await.unwrap();
    db.migrate().await.unwrap();
    let config = Config::new_from_path(config_path).unwrap();
    let manager = Manager::new(&db, &config);
    (db, manager)
}
