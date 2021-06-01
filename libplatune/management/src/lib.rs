use std::time::Instant;

use postage::{prelude::Stream, sink::Sink};

pub async fn traverse() {
    let num_tasks = std::cmp::min(num_cpus::get(), 4);
    let (mut tx, _) = postage::dispatch::channel(10000);
    let tasks = (0..num_tasks)
        .map(|_| {
            let mut rx = tx.subscribe();
            let handle = tokio::spawn(async move {
                while let Some(path) = rx.recv().await {
                    let tag = audiotags::Tag::new().read_from_path(path);
                    match tag {
                        Err(e) => {
                            println!("{:?}", e);
                        }
                        Ok(tag) => {}
                    }
                }
            });
            return handle;
        })
        .collect::<Vec<tokio::task::JoinHandle<()>>>();

    for entry in walkdir::WalkDir::new("/home/aschey/windows/shared_files/Music") {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension();
        if ext.is_none() {
            continue;
        }

        match ext.unwrap().to_str().unwrap() {
            "mp3" | "m4a" => {
                tx.send(path.to_owned()).await.unwrap();
            }
            _ => {}
        }
    }
    println!("done");
    drop(tx);
    for handle in tasks {
        handle.await.unwrap();
    }
}
