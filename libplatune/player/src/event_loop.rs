use std::sync::mpsc::{Receiver, Sender, SyncSender};

use tokio::sync::broadcast;
use tracing::{error, info};

use crate::{
    enums::{Command, PlayerEvent},
    player::Player,
};

pub(crate) fn ended_loop(receiver: Receiver<Receiver<()>>, request_tx: SyncSender<Command>) {
    while let Ok(ended_receiver) = receiver.recv() {
        // Strange platform-specific behavior here
        // On Windows, receiver.recv() always returns Ok, but on Linux it returns Err
        // after the first event if the queue is stopped
        ended_receiver.recv().unwrap_or_default();
        if let Err(e) = request_tx.send(Command::Ended) {
            error!("Error sending song ended message {:?}", e);
        }
    }
}

pub(crate) fn main_loop(
    receiver: Receiver<Command>,
    finish_rx: Sender<Receiver<()>>,
    event_tx: broadcast::Sender<PlayerEvent>,
) {
    let (_stream, handle) = match rodio::OutputStream::try_default() {
        Ok((stream, handle)) => (stream, handle),
        Err(e) => {
            error!("Error creating audio output stream {:?}", e);
            return;
        }
    };

    let mut queue = match Player::new(finish_rx, event_tx, handle) {
        Ok(player) => player,
        Err(e) => {
            error!("Error creating audio sink {:?}", e);
            return;
        }
    };

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
            Command::GetCurrentTime(current_time_tx) => {
                let current_time = queue.get_curent_time();
                current_time_tx.send(current_time);
            }
            Command::Shutdown => {
                return;
            }
        }
        info!("Completed command");
    }
    info!("Request loop completed");
}
