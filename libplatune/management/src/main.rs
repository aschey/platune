use std::time::Instant;

use libplatune_management::traverse;
use postage::{prelude::Stream, sink::Sink};

#[tokio::main]
async fn main() {
    traverse().await;
}
