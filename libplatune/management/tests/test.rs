use std::{
    env,
    fs::{File, OpenOptions},
};

use directories::BaseDirs;
use libplatune_management::{config::Config, database::Database};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test() {
    OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open("test.db")
        .unwrap();
    let db = Database::connect("test.db").await;
    db.migrate().await;
    let dir = env::temp_dir().join("platune_test");
    let config = Config::new_from_path(&db, dir);
    config.add_folder("test1").await;
    config.add_folder("test1").await;
    config.add_folder("test2").await;
    let folders = config.get_all_folders().await;
    assert_eq!(vec!["test1", "test2"], folders);
}
