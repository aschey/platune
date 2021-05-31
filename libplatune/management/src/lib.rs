use std::time::Instant;

use postage::{prelude::Stream, sink::Sink};

pub fn traverse() {
    for entry in walkdir::WalkDir::new("/home/aschey/windows/shared_files/Music") {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension();
        if ext.is_none() {
            println!("none {:?}", path);
            continue;
        }

        match ext.unwrap().to_str().unwrap() {
            "mp3" | "m4a" => {
                let tag = audiotags::Tag::new().read_from_path(path);
                match tag {
                    Err(e) => {
                        println!("{:?} {:?}", path, e);
                    }
                    Ok(tag) => {}
                }
            }
            _ => {}
        }
    }
}
pub async fn traverse_async() {
    let (mut tx, mut rx) = postage::mpsc::channel(10000);
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
    for entry in walkdir::WalkDir::new("/home/aschey/windows/shared_files/Music") {
        let entry = entry.unwrap();
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension();
        if ext.is_none() {
            println!("none {:?}", path);
            continue;
        }

        match ext.unwrap().to_str().unwrap() {
            "mp3" | "m4a" => {
                tx.send(path.to_owned()).await.unwrap();
                // let tag = audiotags::Tag::new().read_from_path(path);
                // match tag {
                //     Err(e) => {
                //         println!("{:?} {:?}", path, e);
                //     }
                //     Ok(tag) => {}
                // }
            }
            _ => {}
        }
    }
    drop(tx);
    handle.await.unwrap();
}
