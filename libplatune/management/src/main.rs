use core::time;
use std::{
    fs::{DirEntry, File},
    io::{self, BufReader, Read},
    path::PathBuf,
    sync::mpsc,
    thread,
    time::Instant,
};

use libplatune_management::traverse;
use postage::{dispatch, prelude::Stream, sink::Sink};
use sqlx::{Connection, Executor, SqliteConnection, SqlitePool};

#[tokio::main]
async fn main() {
    let now = Instant::now();
    traverse().await;

    println!("{:?}", now.elapsed());
}
