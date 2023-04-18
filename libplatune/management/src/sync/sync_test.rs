use crate::{config::MemoryConfig, database::Database, manager::Manager};
use futures::StreamExt;
use itertools::Itertools;
use lofty::{Accessor, ItemKey, Probe, TagExt, TaggedFileExt};
use normpath::PathExt;
use pretty_assertions::assert_eq;
use rstest::*;
use std::{
    fs::{self, create_dir, create_dir_all, File},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tempfile::TempDir;
use tracing::Level;

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
    let (_, mut manager) = setup().await;
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

    assert_eq!(1.0, *msgs.last().unwrap());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_no_folder() {
    let (_, mut manager) = setup().await;

    let mut receiver = manager.sync(None).await.unwrap();
    let mut msgs = vec![];
    while let Some(msg) = receiver.next().await {
        msgs.push(msg.unwrap());
    }

    assert_eq!(Vec::<f32>::new(), msgs);
}

#[rstest(use_mount, case(true), case(false))]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_basic(use_mount: bool) {
    let tempdir = TempDir::new().unwrap();
    let (_, mut manager) = setup().await;
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
    // progress message order is not deterministic so this is the best we can do
    assert_eq!(1.0, *msgs.last().unwrap());
    assert!(!msgs.is_empty());

    for path in paths {
        assert!(manager.get_song_by_path(path).await.unwrap().is_some());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_multiple() {
    let tempdir = TempDir::new().unwrap();
    let (_, mut manager) = setup().await;
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
    let (_, mut manager) = setup().await;
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

    let last_song = inner_dir.join("test3.mp3");
    // Recreate deleted file before normalizing
    create_dir_all(inner_dir).unwrap();
    File::create(&last_song).unwrap();

    assert_eq!(1, deleted.len());
    assert_normalized(
        deleted[0].song_path.clone(),
        last_song.to_string_lossy().to_string(),
    );

    // TODO: add test to check if number of songs decreased once we have an endpoint to get all songs
    assert_eq!(0, deleted2.len());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_delete_and_readd() {
    let tempdir = TempDir::new().unwrap();
    let (_, mut manager) = setup().await;
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

    assert_eq!(1, deleted.len());
    assert_normalized(
        deleted[0].song_path.clone(),
        last_song.to_string_lossy().to_string(),
    );

    // TODO: add test to check if number of songs decreased once we have an endpoint to get all songs
    assert_eq!(0, deleted2.len());
}

#[rstest(do_update, case(true), case(false))]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn test_sync_duplicate_album_name(do_update: bool) {
    let tempdir = TempDir::new().unwrap();
    let (_, mut manager) = setup().await;
    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    let song1_path = inner_dir.join("test.mp3");
    let song2_path = inner_dir.join("test2.mp3");
    fs::copy("../test_assets/test.mp3", &song1_path).unwrap();
    fs::copy("../test_assets/test2.mp3", &song2_path).unwrap();
    {
        let mut track1 = Probe::open(&song1_path).unwrap().read().unwrap();
        let tag1 = track1.primary_tag_mut().unwrap();
        tag1.set_album("album".to_owned());
        tag1.insert_text(ItemKey::AlbumArtist, "artist1".to_owned());
        tag1.set_title("track1".to_owned());
        tag1.save_to_path(&song1_path).unwrap();
    }

    {
        let mut track2 = Probe::open(&song2_path).unwrap().read().unwrap();
        let tag2 = track2.primary_tag_mut().unwrap();
        tag2.set_album("album".to_owned());
        if do_update {
            tag2.insert_text(ItemKey::AlbumArtist, "artist1".to_owned());
        } else {
            tag2.insert_text(ItemKey::AlbumArtist, "artist2".to_owned());
        }

        tag2.set_title("track2".to_owned());
        tag2.save_to_path(&song2_path).unwrap();
    }

    manager
        .add_folder(music_dir.to_str().unwrap())
        .await
        .unwrap();

    let mut receiver = manager.sync(None).await.unwrap();
    while (receiver.next().await).is_some() {}

    if do_update {
        {
            let mut track2 = Probe::open(&song2_path).unwrap().read().unwrap();
            let tag2 = track2.primary_tag_mut().unwrap();
            tag2.insert_text(ItemKey::AlbumArtist, "artist2".to_owned());
            tag2.save_to_path(&song2_path).unwrap();
        }

        let mut receiver = manager.sync(None).await.unwrap();
        while (receiver.next().await).is_some() {}
    }

    let song1_entry = manager
        .get_song_by_path(&song1_path)
        .await
        .unwrap()
        .unwrap();
    assert_eq!("album", song1_entry.album);
    assert_eq!("artist1", song1_entry.album_artist);
    assert_eq!("track1", song1_entry.song);

    let song2_entry = manager
        .get_song_by_path(&song2_path)
        .await
        .unwrap()
        .unwrap();
    assert_eq!("album", song2_entry.album);
    assert_eq!("artist2", song2_entry.album_artist);
    assert_eq!("track2", song2_entry.song);
}

async fn setup() -> (Database, Manager) {
    let db = Database::connect_in_memory().await.unwrap();
    db.sync_database().await.unwrap();
    let config = Arc::new(MemoryConfig::new_boxed());
    let manager = Manager::new(&db, config.clone());
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

fn assert_normalized(left: String, right: String) {
    assert_eq!(normalize(&left), normalize(&right));
}
