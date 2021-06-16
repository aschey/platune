#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("features 'runtime-tokio' and 'runtime-async-std' are mutually exclusive");

mod enums;
mod event_loop;
#[cfg(feature = "runtime-tokio")]
mod http_stream_reader;
mod player;
pub mod platune_player {
    pub use crate::enums::{Command, PlayerEvent};
    use crate::event_loop::{ended_loop, main_loop};
    pub use postage::*;

    pub use postage::{sink::Sink, stream::Stream};
    use std::fs::remove_file;

    pub struct PlatunePlayer {
        cmd_sender: std::sync::mpsc::SyncSender<Command>,
    }

    impl PlatunePlayer {
        pub fn new() -> (PlatunePlayer, broadcast::Receiver<PlayerEvent>) {
            for entry in std::env::temp_dir()
                .read_dir()
                .expect("read_dir call failed")
            {
                if let Ok(entry) = entry {
                    if entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .starts_with("platunecache")
                    {
                        remove_file(entry.path()).unwrap();
                    }
                }
            }
            let (event_tx, event_rx) = broadcast::channel(32);
            let (tx, rx) = std::sync::mpsc::channel();
            let tx_ = tx.clone();
            let (finish_tx, finish_rx) = std::sync::mpsc::sync_channel(32);
            let finish_tx_ = finish_tx.clone();

            let main_loop_fn = || main_loop(finish_rx, tx_, event_tx);
            let ended_loop_fn = || ended_loop(rx, finish_tx_);
            #[cfg(feature = "runtime-tokio")]
            {
                tokio::task::spawn_blocking(main_loop_fn);
                tokio::task::spawn_blocking(ended_loop_fn);
            }

            (
                PlatunePlayer {
                    cmd_sender: finish_tx,
                },
                event_rx,
            )
        }

        pub fn set_queue(&self, queue: Vec<String>) {
            self.cmd_sender.send(Command::SetQueue(queue)).unwrap();
        }

        pub fn add_to_queue(&self, song: String) {
            self.cmd_sender.send(Command::AddToQueue(song)).unwrap();
        }

        pub fn seek(&self, millis: u64) {
            self.cmd_sender.send(Command::Seek(millis)).unwrap();
        }

        pub fn start(&self) {
            self.cmd_sender.send(Command::Start).unwrap();
        }

        pub fn stop(&self) {
            self.cmd_sender.send(Command::Stop).unwrap();
        }

        pub fn set_volume(&self, volume: f32) {
            self.cmd_sender.send(Command::SetVolume(volume)).unwrap();
        }

        pub fn pause(&self) {
            self.cmd_sender.send(Command::Pause).unwrap();
        }

        pub fn resume(&self) {
            self.cmd_sender.send(Command::Resume).unwrap();
        }

        pub fn next(&self) {
            self.cmd_sender.send(Command::Next).unwrap();
        }

        pub fn previous(&self) {
            self.cmd_sender.send(Command::Previous).unwrap();
        }

        pub fn join(&self) {
            self.cmd_sender.send(Command::Shutdown).unwrap();
        }
    }

    impl Drop for PlatunePlayer {
        fn drop(&mut self) {
            self.cmd_sender.send(Command::Shutdown).unwrap();
        }
    }
}
