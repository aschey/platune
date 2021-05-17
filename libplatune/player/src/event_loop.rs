use std::{
    io::BufReader,
    sync::mpsc::{Receiver, Sender},
};

use log::info;
use rodio::Sink;

use crate::sink_actor::SinkActor;

pub fn ended_loop(receiver: Receiver<Receiver<()>>, request_tx: Sender<Command>) {
    while let Ok(receiver) = receiver.recv() {
        if receiver.recv().is_ok() {
            request_tx.send(Command::Ended).unwrap();
        }
    }
}

pub fn start_loop(receiver: Receiver<Command>, finish_rx: Sender<Receiver<()>>) {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let mut queue = SinkActor::new(finish_rx, handle);
    while let Ok(next_command) = receiver.recv() {
        info!("Got command {:?}", next_command);
        match next_command {
            Command::SetQueue(songs) => {
                queue.set_queue(songs);
            }
            Command::Seek(seconds) => {
                queue.seek(seconds);
            }
            Command::SetVolume(volume) => {
                queue.set_volume(volume);
            }
            Command::Pause => {
                queue.pause();
            }
            Command::Start => {
                queue.start();
            }
            Command::Resume => {
                queue.play();
            }
            Command::Stop => {
                queue.stop();
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
