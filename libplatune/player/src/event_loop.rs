use std::{
    io::BufReader,
    sync::mpsc::{Receiver, Sender},
};

use log::info;
use rodio::Sink;

use crate::sink_actor::SinkActor;

pub fn ended_loop(receiver: Receiver<Receiver<()>>, request_tx: Sender<Command>) {
    while let Ok(receiver) = receiver.recv() {
        receiver.recv().unwrap();
        request_tx.send(Command::Ended).unwrap();
    }
}

pub fn start_loop(
    sender: Sender<Command>,
    receiver: Receiver<Command>,
    finish_rx: Sender<Receiver<()>>,
) {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    let mut queue = SinkActor::new(sender, finish_rx, sink);
    while let Ok(next_command) = receiver.recv() {
        info!("Got command {:?}", next_command);
        match next_command {
            Command::SetQueue(songs) => {
                queue.set_queue(songs);
            }
            Command::Seek(seconds) => {
                //call!(self.queue_addr.seek(seconds)).await.unwrap();
            }
            Command::SetVolume(volume) => {
                //call!(self.player_addr.set_volume(volume)).await.unwrap();
            }
            Command::Pause => {
                //call!(self.player_addr.pause()).await.unwrap();
            }
            Command::Start => {
                queue.start();
            }
            Command::Resume => {
                //call!(self.player_addr.resume()).await.unwrap();
            }
            Command::Stop => {
                //call!(self.player_addr.stop()).await.unwrap();
            }
            Command::Ended => {
                queue.on_ended();
            }
            Command::Next => {
                //call!(self.queue_addr.next()).await.unwrap();
            }
            Command::Previous => {
                // call!(self.queue_addr.previous()).await.unwrap();
            }
            Command::Shutdown => {
                // call!(self.player_addr.shutdown()).await.unwrap();
            }
        }
        info!("Completed command");
    }
    info!("Request loop completed");
}

#[derive(Debug, Clone)]
pub enum Command {
    SetQueue(Vec<String>),
    Seek(u64),
    SetVolume(f32),
    Pause,
    Resume,
    Start,
    Stop,
    Ended,
    Next,
    Previous,
    Shutdown,
}
