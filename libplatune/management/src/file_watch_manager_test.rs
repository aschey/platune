use super::{FileWatchManager, Progress};
use crate::{config::MemoryConfig, database::Database, manager::Manager};
use itertools::Itertools;
use pretty_assertions::assert_eq;
use rstest::*;
use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};
use tempfile::{tempdir, TempDir};
use tokio::{sync::mpsc, time::timeout};
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

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_file_sync_sequential(
    #[values(true, false)] add_folder_before: bool,
    #[values(true, false)] rename_dir: bool,
    #[values(true, false)] rename_file: bool,
    #[values(1, 100)] debounce_time: u64,
) {
    let (_tempdir, temp_path) = create_tempdir();
    let (_, manager) = setup().await;

    let music_dir = temp_path.join("configdir");
    let mut inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    if add_folder_before {
        // Test that folder gets added to watcher
        manager
            .add_folder(music_dir.to_str().unwrap())
            .await
            .unwrap();
    }

    let file_watch_manager = FileWatchManager::new(manager, Duration::from_millis(debounce_time))
        .await
        .unwrap();
    let mut receiver = file_watch_manager.subscribe_progress();

    if !add_folder_before {
        // Test that watcher picks up existing folder
        file_watch_manager
            .add_folder(music_dir.to_str().unwrap())
            .await
            .unwrap();
    }

    let msg_task = tokio::spawn(async move {
        // Wait for all syncs to finish
        while timeout(Duration::from_secs(1), receiver.recv())
            .await
            .is_ok()
        {}
    });

    let mut paths = vec![
        inner_dir.join("test.mp3"),
        inner_dir.join("test2.mp3"),
        inner_dir.join("test3.mp3"),
    ];
    fs::copy("../test_assets/test.mp3", &paths[0]).unwrap();
    fs::copy("../test_assets/test2.mp3", &paths[1]).unwrap();
    fs::copy("../test_assets/test3.mp3", &paths[2]).unwrap();

    // Wait until sync finished
    timeout(Duration::from_secs(5), msg_task)
        .await
        .unwrap()
        .unwrap();

    // Can't hold a readable lock outside this block because a writable lock is required to sync
    {
        // Make sure all files synced
        let manager = file_watch_manager.read().await;
        for path in &paths {
            assert!(manager.get_song_by_path(path).await.unwrap().is_some());
        }
    }

    let mut msg_task = None;
    if rename_dir || rename_file {
        let mut receiver = file_watch_manager.subscribe_progress();
        msg_task = Some(tokio::spawn(async move {
            // Wait for all syncs to finish
            while timeout(Duration::from_secs(1), receiver.recv())
                .await
                .is_ok()
            {}
        }));
    }

    if rename_dir {
        let new_dir = music_dir.join("folder2");
        fs::rename(&inner_dir, &new_dir).unwrap();
        inner_dir = new_dir;
        paths = vec![
            inner_dir.join("test.mp3"),
            inner_dir.join("test2.mp3"),
            inner_dir.join("test3.mp3"),
        ];
    }

    if rename_file {
        let new_path = inner_dir.join("test4.mp3");
        fs::rename(&paths[0], &new_path).unwrap();
        paths[0] = new_path;
    }

    if rename_dir || rename_file {
        // Wait for second sync to finish
        timeout(Duration::from_secs(5), msg_task.unwrap())
            .await
            .unwrap()
            .unwrap();
        let manager = file_watch_manager.read().await;

        // Make sure paths got upated
        for path in paths {
            assert!(manager.get_song_by_path(path).await.unwrap().is_some());
        }
    }
}

#[rstest(rename, case(true), case(false))]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_file_sync_concurrent(rename: bool) {
    let (_tempdir, temp_path) = create_tempdir();
    let (_, manager) = setup().await;

    let music_dir = temp_path.join("configdir");
    let inner_dir = music_dir.join("folder1");

    let file_watch_manager = FileWatchManager::new(manager, Duration::from_millis(10))
        .await
        .unwrap();
    let mut receiver = file_watch_manager.subscribe_progress();

    let (started_tx, mut started_rx) = mpsc::channel(1);
    let msg_task = tokio::spawn(async move {
        // Wait for all syncs to finish
        while timeout(Duration::from_secs(5), receiver.recv())
            .await
            .is_ok()
        {
            started_tx.try_send(()).unwrap_or_default();
        }
    });

    let mut paths = vec![];
    for i in 0..100 {
        let new_dir = inner_dir.clone().join(format!("test{i}"));
        let new_dir_str = new_dir.to_string_lossy().to_string();
        create_dir_all(new_dir).unwrap();

        file_watch_manager.add_folder(&new_dir_str).await.unwrap();

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

    started_rx.recv().await;
    // Make second file change as soon as first sync starts, don't wait for it to complete
    let new_path = inner_dir.join("test0/test4.mp3");
    if rename {
        fs::rename(&paths[0], &new_path).unwrap();
        paths[0] = new_path;
    } else {
        let new_path = inner_dir.join("test0/test4.mp3");
        fs::copy("../test_assets/test.mp3", &new_path).unwrap();
        paths.push(new_path);
    }

    timeout(Duration::from_secs(20), msg_task)
        .await
        .unwrap()
        .unwrap();

    let manager = file_watch_manager.read().await;

    for path in &paths {
        assert!(manager.get_song_by_path(path).await.unwrap().is_some());
    }
}

#[rstest(sync_twice, case(true), case(false))]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_sync_all(sync_twice: bool) {
    let (_tempdir, temp_path) = create_tempdir();
    let (_, manager) = setup().await;

    let music_dir = temp_path.join("configdir");
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

    let file_watch_manager = FileWatchManager::new(manager, Duration::from_millis(100))
        .await
        .unwrap();
    let mut receiver = file_watch_manager.subscribe_progress();

    let msg_task = tokio::spawn(async move {
        while let Ok(Progress {
            finished: false, ..
        }) = receiver.recv().await
        {}
    });

    let mut msg_task2 = None;
    if sync_twice {
        let mut receiver2 = file_watch_manager.subscribe_progress();

        msg_task2 = Some(tokio::spawn(async move {
            while let Ok(Progress {
                finished: false, ..
            }) = receiver2.recv().await
            {}
        }));
    }

    file_watch_manager.start_sync_all().await.unwrap();

    if sync_twice {
        timeout(Duration::from_secs(5), msg_task2.unwrap())
            .await
            .unwrap()
            .unwrap();
    }

    timeout(Duration::from_secs(5), msg_task)
        .await
        .unwrap()
        .unwrap();

    let manager = file_watch_manager.read().await;
    for path in &paths {
        assert!(manager.get_song_by_path(path).await.unwrap().is_some());
    }
}

#[rstest(paths, new_path, expected,
    case(vec![], "/test/path/1", vec!["/test/path/1"]),
    case(vec!["/test/path/1"], "/test/path/2", vec!["/test/path/1", "/test/path/2"]),
    case(vec!["/test/path"], "/test/path/1", vec!["/test/path"]),
    case(vec!["/test/path/1"], "/test/path", vec!["/test/path"]),
    case(vec!["/test/path/1","/test/path/2"], "/test", vec!["/test"]),
    case(vec!["/test/path/1"], "/test/path/1", vec!["/test/path/1"]))]
fn test_normalize(paths: Vec<&str>, new_path: &str, expected: Vec<&str>) {
    let new_paths = FileWatchManager::normalize_paths(
        paths.into_iter().map(PathBuf::from).collect(),
        PathBuf::from(new_path),
    );
    let expected = expected.into_iter().map(PathBuf::from).collect_vec();
    assert_eq!(expected, new_paths);
}

fn create_tempdir() -> (TempDir, PathBuf) {
    let temp = tempdir().unwrap();
    let mut temp_path = temp.path().to_owned();
    create_dir_all(&temp_path).unwrap();
    // Tempdir in mac defaults to a symlink so we need to remove the symlink so the paths all match
    if cfg!(target_os = "macos") {
        temp_path = std::fs::canonicalize(temp_path).unwrap();
    }
    (temp, temp_path)
}

async fn setup() -> (Database, Manager) {
    let db = Database::connect_in_memory().await.unwrap();
    db.sync_database().await.unwrap();
    let config = Arc::new(MemoryConfig::new_boxed());
    let manager = Manager::new(&db, config);

    (db, manager)
}
