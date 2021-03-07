use act_zero::{call, Actor, Addr};
use log::info;
use postage::{mpsc::Receiver, prelude::Stream};

use super::{player::Player, song_queue::SongQueue};

pub struct RequestHandler {
    request_queue: Receiver<Command>,
    queue_addr: Addr<SongQueue>,
    player_addr: Addr<Player>,
}

impl Actor for RequestHandler {}

impl RequestHandler {
    pub fn new(
        request_queue: Receiver<Command>,
        queue_addr: Addr<SongQueue>,
        player_addr: Addr<Player>,
    ) -> RequestHandler {
        RequestHandler {
            request_queue,
            queue_addr,
            player_addr,
        }
    }
    pub async fn run(&mut self) {
        while let Some(next_command) = self.request_queue.recv().await {
            info!("Got command {:#?}", next_command);
            match next_command {
                Command::SetQueue(queue) => {
                    call!(self.queue_addr.set_queue(queue)).await.unwrap();
                }
                Command::Seek(seconds) => {
                    call!(self.player_addr.seek(seconds)).await.unwrap();
                }
                Command::SetVolume(volume) => {
                    call!(self.player_addr.set_volume(volume)).await.unwrap();
                }
                Command::Pause => {
                    call!(self.player_addr.pause()).await.unwrap();
                }
                Command::Resume => {
                    call!(self.player_addr.resume()).await.unwrap();
                }
                Command::Stop => {
                    call!(self.player_addr.stop()).await.unwrap();
                }
                Command::Ended => {
                    call!(self.queue_addr.on_ended()).await.unwrap();
                }
            }
            info!("Completed command");
        }
        info!("Request loop completed");
    }
}
#[derive(Debug, Clone)]
pub enum Command {
    SetQueue(Vec<String>),
    Seek(f64),
    SetVolume(f32),
    Pause,
    Resume,
    Stop,
    Ended,
}
