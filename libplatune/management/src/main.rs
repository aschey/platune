use std::time::Instant;

use libplatune_management::{traverse, traverse_async};
use postage::{prelude::Stream, sink::Sink};

#[tokio::main]
async fn main() {
    let (mut tx, mut rx) = postage::mpsc::channel(10000);
    tx.try_send(1).unwrap();
    drop(tx);
    let res = rx.recv().await.unwrap();
    println!("{}", res);
}
