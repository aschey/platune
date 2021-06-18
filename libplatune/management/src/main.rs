use std::time::Instant;

use libplatune_management::{config::Config, database::Database};

#[tokio::main]
async fn main() {
    dotenv::from_path("./.env").unwrap_or_default();
    let path = std::env::var("DATABASE_URL")
        .unwrap()
        .replace("sqlite://", "");
    let db = Database::connect(path).await;
    let config = Config::new(&db);
    config.register_drive("test").await;

    println!("{}", config.get_drive_id().unwrap());

    let now = Instant::now();
    let mut rx = db.sync();
    while let Some(_) = rx.recv().await {}

    println!("{:?}", now.elapsed());
}
