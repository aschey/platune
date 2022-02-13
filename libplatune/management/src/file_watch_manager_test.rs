use super::FileWatchManager;
use crate::{config::Config, database::Database, manager::Manager};
use std::{
    fs::{self, create_dir_all},
    time::Duration,
};
use tempfile::TempDir;
use tokio::time::timeout;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_file_sync() {
    let tempdir = TempDir::new().unwrap();
    let (db, manager) = setup(&tempdir).await;

    let music_dir = tempdir.path().join("configdir");
    let inner_dir = music_dir.join("folder1");
    create_dir_all(inner_dir.clone()).unwrap();

    manager
        .add_folder(music_dir.to_str().unwrap())
        .await
        .unwrap();

    let file_watch_manager = FileWatchManager::new(manager).await;
    let mut receiver = file_watch_manager.subscribe_progress();

    let msg_task = tokio::spawn(async move {
        while let Ok(msg) = receiver.recv().await {
            if msg.finished {
                break;
            }
        }
    });

    let paths = vec![
        inner_dir.join("test.mp3"),
        inner_dir.join("test2.mp3"),
        inner_dir.join("test3.mp3"),
    ];
    fs::copy("../test_assets/test.mp3", &paths[0]).unwrap();
    fs::copy("../test_assets/test2.mp3", &paths[1]).unwrap();
    fs::copy("../test_assets/test3.mp3", &paths[2]).unwrap();
    msg_task.await.unwrap();

    let manager = file_watch_manager.read().await;
    for path in paths {
        assert!(manager.get_song_by_path(path).await.unwrap().is_some());
    }

    timeout(Duration::from_secs(5), db.close())
        .await
        .unwrap_or_default();
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
