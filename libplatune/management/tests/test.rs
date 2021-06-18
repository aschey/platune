use std::{
    env,
    fs::{File, OpenOptions},
    time::Duration,
};

use directories::BaseDirs;
use libplatune_management::{config::Config, database::Database};
use log::LevelFilter;
use sqlx::{
    pool::PoolConnection,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    ConnectOptions, Sqlite, SqlitePool,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
pub async fn test() {
    let (db, conn) = get_db().await;
    let dir = env::temp_dir().join("platune_test1");
    let config = Config::new_from_path(&db, dir);
    config.add_folder("test1").await;
    config.add_folder("test1").await;
    config.add_folder("test2").await;
    let folders = config.get_all_folders().await;
    assert_eq!(vec!["test1", "test2"], folders);
    conn.release();
}

async fn get_db() -> (Database, PoolConnection<Sqlite>) {
    let opts = SqliteConnectOptions::new()
        .filename(":memory:")
        .log_statements(LevelFilter::Debug)
        .to_owned();

    let pool = SqlitePool::connect_with(opts).await.unwrap();
    let conn = pool.acquire().await.unwrap();
    let db = Database::from_pool(pool).await;
    db.migrate().await;
    (db, conn)
}
