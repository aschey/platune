use act_zero::{call, Actor, Addr};
use futures::{channel::mpsc::Receiver, StreamExt};
use log::info;

use super::song_queue::SongQueue;

pub struct RequestHandler {
    request_queue: Receiver<Command>,
    queue_addr: Addr<SongQueue>,
}

impl Actor for RequestHandler {}

impl RequestHandler {
    pub fn new(request_queue: Receiver<Command>, queue_addr: Addr<SongQueue>) -> RequestHandler {
        RequestHandler {
            request_queue,
            queue_addr,
        }
    }
    pub async fn run(&mut self) {
        while let Some(next_command) = self.request_queue.next().await {
            info!("Got cmd {:#?}", next_command);
            match next_command {
                Command::SetQueue(queue) => {
                    call!(self.queue_addr.set_queue(queue)).await.unwrap();
                }
            }
            info!("Completed cmd");
        }
    }
}
#[derive(Debug)]
pub enum Command {
    SetQueue(Vec<String>),
}
