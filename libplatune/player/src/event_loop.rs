use std::{
    io::BufReader,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use log::info;
use rodio::Sink;

use crate::{libplayer::PlayerEvent, player::Player};

pub fn ended_loop(receiver: Receiver<Receiver<()>>, request_tx: Sender<Command>) {
    while let Ok(receiver) = receiver.recv() {
        match receiver.recv() {
            Ok(_) => {
                request_tx.send(Command::Ended).unwrap();
            }
            Err(_) => {
                info!("Ended receiver disconnected");
            }
        }
    }
}

pub fn start_loop(
    receiver: Receiver<Command>,
    finish_rx: Sender<Receiver<()>>,
    event_tx: postage::broadcast::Sender<PlayerEvent>,
) {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let mut queue = Player::new(finish_rx, event_tx, handle);
    while let Ok(next_command) = receiver.recv() {
        info!("Got command {:?}", next_command);
        match next_command {
            Command::SetQueue(songs) => {
                queue.set_queue(songs);
            }
            Command::Seek(millis) => {
                queue.seek(millis);
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
                queue.go_next();
            }
            Command::Previous => {
                queue.go_previous();
            }
            Command::Shutdown => {
                return;
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
