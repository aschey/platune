use std::{
    io::BufReader,
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

use libplatune_player::libplayer::PlatunePlayer;
use log::info;

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

pub fn start_loop(receiver: Receiver<Command>) {
    let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&handle).unwrap();
    while let Ok(next_command) = receiver.recv() {
        info!("Got command {:?}", next_command);
        match next_command {
            Command::SetQueue(songs) => {
                //queue.set_queue(songs);
            }
            Command::Seek(seconds) => {
                //call!(self.queue_addr.seek(seconds)).await.unwrap();
            }
            Command::SetVolume(volume) => {
                //call!(self.player_addr.set_volume(volume)).await.unwrap();
            }
            Command::Pause => {
                sink.pause();
                //call!(self.player_addr.pause()).await.unwrap();
            }
            Command::Start => {
                let file =
                    std::fs::File::open("C:\\shared_files\\Music\\EDM Mixes\\April - 2013.mp3")
                        .unwrap();
                let file2 = std::fs::File::open(
                    "C:\\shared_files\\Music\\Between the Buried and Me\\Colors\\05 Ants of the Sky.m4a",
                )
                .unwrap();
                let decoder1 = rodio::Decoder::new(BufReader::new(file)).unwrap();
                let mut decoder2 = rodio::Decoder::new(BufReader::new(file2)).unwrap();
                //S.buffered();
                //sink.append(decoder1);
                sink.append(decoder2);
            }
            Command::Resume => {
                //call!(self.player_addr.resume()).await.unwrap();
            }
            Command::Stop => {
                //call!(self.player_addr.stop()).await.unwrap();
            }
            Command::Ended => {
                //queue.on_ended();
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

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel();
    tokio::task::spawn_blocking(move || start_loop(rx));
    tx.send(Command::Start).unwrap();
    thread::sleep(Duration::from_secs(2));
    tx.send(Command::Pause).unwrap();
    thread::sleep(Duration::from_secs(2));
}
