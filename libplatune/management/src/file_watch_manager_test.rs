use super::{FileWatchManager, Progress};
use crate::{config::Config, database::Database, manager::Manager};
use itertools::Itertools;
use pretty_assertions::assert_eq;
use rstest::*;
use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
    time::Duration,
};
use tempfile::TempDir;
use tokio::time::timeout;

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_file_sync(
    #[values(true, false)] add_folder_before: bool,
    #[values(true, false)] rename_dir: bool,
    #[values(true, false)] rename_file: bool,
) {
    let tempdir = TempDir::new().unwrap();
    let (db, manager) = setup(&tempdir).await;

    let music_dir = tempdir.path().join("configdir");
    let mut inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    if add_folder_before {
        manager
            .add_folder(music_dir.to_str().unwrap())
            .await
            .unwrap();
    }

    let file_watch_manager = FileWatchManager::new(manager, Duration::from_millis(100)).await;
    let mut receiver = file_watch_manager.subscribe_progress();

    if !add_folder_before {
        file_watch_manager
            .add_folder(music_dir.to_str().unwrap())
            .await
            .unwrap();
    }

    let msg_task = tokio::spawn(async move {
        while let Ok(Progress {
            finished: false, ..
        }) = receiver.recv().await
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
    timeout(Duration::from_secs(5), msg_task)
        .await
        .unwrap()
        .unwrap();

    // Can't hold a readable lock outside this block because a writable lock is required to sync
    {
        let manager = file_watch_manager.read().await;
        for path in &paths {
            assert!(manager.get_song_by_path(path).await.unwrap().is_some());
        }
    }

    let mut msg_task = None;
    if rename_dir || rename_file {
        let mut receiver = file_watch_manager.subscribe_progress();
        msg_task = Some(tokio::spawn(async move {
            while let Ok(Progress {
                finished: false, ..
            }) = receiver.recv().await
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
        timeout(Duration::from_secs(5), msg_task.unwrap())
            .await
            .unwrap()
            .unwrap();
        let manager = file_watch_manager.read().await;

        for path in paths {
            assert!(manager.get_song_by_path(path).await.unwrap().is_some());
        }
    }

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_sync_all() {
    let tempdir = TempDir::new().unwrap();
    let (db, manager) = setup(&tempdir).await;

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

    let file_watch_manager = FileWatchManager::new(manager, Duration::from_millis(100)).await;
    let mut receiver = file_watch_manager.subscribe_progress();

    let msg_task = tokio::spawn(async move {
        while let Ok(Progress {
            finished: false, ..
        }) = receiver.recv().await
        {}
    });

    file_watch_manager.start_sync_all().await;

    timeout(Duration::from_secs(5), msg_task)
        .await
        .unwrap()
        .unwrap();

    let manager = file_watch_manager.read().await;
    for path in &paths {
        assert!(manager.get_song_by_path(path).await.unwrap().is_some());
    }

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();
}

#[rstest(paths, new_path, expected,
    case(vec![], "/test/path/1", vec!["/test/path/1"]),
    case(vec!["/test/path/1"], "/test/path/2", vec!["/test/path/1", "/test/path/2"]),
    case(vec!["/test/path"], "/test/path/1", vec!["/test/path"]),
    case(vec!["/test/path/1"], "/test/path", vec!["/test/path"]),
    case(vec!["/test/path/1","/test/path/2"], "/test", vec!["/test"]))]
fn test_normalize(paths: Vec<&str>, new_path: &str, expected: Vec<&str>) {
    let new_paths = FileWatchManager::normalize_paths(
        paths.into_iter().map(PathBuf::from).collect(),
        PathBuf::from(new_path),
    );
    let expected = expected.into_iter().map(PathBuf::from).collect_vec();
    assert_eq!(expected, new_paths);
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
