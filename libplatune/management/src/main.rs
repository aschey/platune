use std::time::Instant;

use libplatune_management::traverse;
use postage::{prelude::Stream, sink::Sink};
use sqlx::{Connection, Executor, SqliteConnection, SqlitePool};

#[tokio::main]
async fn main() {
    dotenv::from_path("./.env").unwrap_or_default();
    let mut pool = SqlitePool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let res = sqlx::query!("select * from artist")
        .fetch_all(&pool)
        .await
        .unwrap();
}
