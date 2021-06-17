use core::time;
use std::{
    fs::{DirEntry, File},
    io::{self, BufReader, Read},
    path::PathBuf,
    sync::mpsc,
    thread,
    time::Instant,
};

use libplatune_management::database::Database;
use postage::{dispatch, prelude::Stream, sink::Sink};
use sqlx::{Connection, Executor, SqliteConnection, SqlitePool};

#[tokio::main]
async fn main() {
    dotenv::from_path("./.env").unwrap_or_default();
    let path = std::env::var("DATABASE_URL")
        .unwrap()
        .replace("sqlite://", "");
    let db = Database::connect(path).await;
    let now = Instant::now();
    let mut rx = db.sync();
    while let Some(res) = rx.recv().await {}

    println!("{:?}", now.elapsed());
}
