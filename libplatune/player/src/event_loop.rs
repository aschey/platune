use std::sync::mpsc::{Receiver, Sender};

use log::info;

use crate::{
    enums::{Command, PlayerEvent},
    player::Player,
};

pub(crate) fn ended_loop(receiver: Receiver<Receiver<()>>, request_tx: Sender<Command>) {
    while let Ok(receiver) = receiver.recv() {
        // Strange platform-specific behavior here
        // On Windows, receiver.recv() always returns Ok, but on Linux it returns Err
        // after the first event if the queue is stopped
        receiver.recv().unwrap_or_default();
        request_tx.send(Command::Ended).unwrap();
    }
}

pub(crate) fn main_loop(
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
            Command::AddToQueue(song) => {
                queue.add_to_queue(song);
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
