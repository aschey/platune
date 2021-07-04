use std::{ops::Range, time::Instant};

use libplatune_management::{config::Config, database::Database};

#[tokio::main]
async fn main() {
    dotenv::from_path("./.env").unwrap_or_default();
    let path = std::env::var("DATABASE_URL")
        .unwrap()
        .replace("sqlite://", "");
    let db = Database::connect(path, true).await;
    //db.migrate().await;
    let now = Instant::now();

    println!("{:?}", db.search("tdagarb", 10).await);

    let config = Config::new(&db);
    // #[cfg(target_os = "windows")]
    // {
    //     config.register_drive("C:\\").await;
    //     config.add_folder("C:\\shared_files\\Music").await;
    // }
    // #[cfg(target_os = "linux")]
    // {
    //     config.register_drive("/home/aschey/windows").await;
    //     config
    //         .add_folder("/home/aschey/windows/shared_files/Music")
    //         .await;
    // }

    //let now = Instant::now();
    // let mut rx = config.sync().await;
    // while let Some(_) = rx.recv().await {}

    println!("{:?}", now.elapsed());
}
