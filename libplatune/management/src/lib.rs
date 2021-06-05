use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use postage::{
    dispatch::{self, Receiver, Sender},
    mpsc,
    prelude::Stream,
    sink::Sink,
};
use tokio::{task::JoinHandle, time::timeout};

pub async fn traverse() {
    controller("C:\\shared_files\\Music".to_owned()).await;
}

async fn controller(path: String) {
    let mut num_tasks = 1;
    let max_tasks = 100;
    let (mut dispatch_tx, _) = dispatch::channel(10000);
    let (finished_tx, mut finished_rx) = mpsc::channel(10000);
    let mut handles = vec![];
    for _ in 0..num_tasks {
        handles.push(spawn_task(
            dispatch_tx.clone(),
            dispatch_tx.subscribe(),
            finished_tx.clone(),
        ));
    }
    dispatch_tx.send(Some(PathBuf::from(path))).await.unwrap();
    let mut dirs = 0;
    loop {
        match timeout(Duration::from_nanos(1), finished_rx.recv()).await {
            Ok(Some(DirRead::Completed)) => {
                dirs -= 1;
                if dirs == 0 {
                    break;
                }
            }
            Ok(Some(DirRead::Found)) => {
                dirs += 1;
            }
            Ok(None) => {
                break;
            }
            Err(_) => {
                println!("spawning task");

                if num_tasks < max_tasks {
                    handles.push(spawn_task(
                        dispatch_tx.clone(),
                        dispatch_tx.subscribe(),
                        finished_tx.clone(),
                    ));
                    num_tasks += 1;
                }
            }
        }
    }
    for _ in 0..handles.len() {
        dispatch_tx.send(None).await.unwrap();
    }
    for handle in handles {
        handle.await.unwrap();
    }
}

fn spawn_task(
    mut dispatch_tx: Sender<Option<PathBuf>>,
    mut dispatch_rx: Receiver<Option<PathBuf>>,
    mut finished_tx: mpsc::Sender<DirRead>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(path) = dispatch_rx.recv().await {
            match path {
                Some(path) => {
                    for dir_result in path.read_dir().unwrap() {
                        let dir = dir_result.unwrap();

                        if dir.file_type().unwrap().is_file() {
                            let name = dir.path();
                            let name = name.extension().unwrap_or_default();
                            match name.to_str().unwrap_or_default() {
                                "mp3" | "m4a" => {
                                    let tag = audiotags::Tag::new().read_from_path(dir.path());
                                    match tag {
                                        Err(e) => {
                                            println!("{:?}", e);
                                        }
                                        Ok(tag) => {}
                                    }
                                }

                                _ => {}
                            }
                        } else {
                            dispatch_tx.send(Some(dir.path())).await.unwrap();
                            finished_tx.send(DirRead::Found).await.unwrap();
                        }
                    }
                    finished_tx.send(DirRead::Completed).await.unwrap();
                }
                None => {
                    break;
                }
            }
        }
    })
}

#[derive(Debug)]
enum DirRead {
    Found,
    Completed,
}
