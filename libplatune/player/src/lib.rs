#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("features 'runtime-tokio' and 'runtime-async-std' are mutually exclusive");

mod enums;
mod event_loop;
#[cfg(feature = "runtime-tokio")]
mod http_stream_reader;
mod player;
pub mod platune_player {
    use tokio::sync::broadcast;
    use tracing::{error, warn};

    pub use crate::enums::PlayerEvent;
    pub use crate::enums::PlayerState;
    use crate::{
        enums::Command,
        event_loop::{ended_loop, main_loop},
    };
    use std::fs::remove_file;

    #[derive(Debug, Clone)]
    pub struct SendError(String);

    #[derive(Debug)]
    pub struct PlatunePlayer {
        cmd_sender: std::sync::mpsc::SyncSender<Command>,
        event_tx: broadcast::Sender<PlayerEvent>,
    }

    impl Default for PlatunePlayer {
        fn default() -> Self {
            Self::new()
        }
    }

    impl PlatunePlayer {
        pub fn new() -> Self {
            Self::clean_temp_files();

            let (event_tx, _) = broadcast::channel(32);
            let (tx, rx) = std::sync::mpsc::channel();
            let tx_ = tx;
            let (finish_tx, finish_rx) = std::sync::mpsc::sync_channel(32);
            let finish_tx_ = finish_tx.clone();

            let event_tx_ = event_tx.clone();
            let main_loop_fn = || main_loop(finish_rx, tx_, event_tx_);
            let ended_loop_fn = || ended_loop(rx, finish_tx_);
            #[cfg(feature = "runtime-tokio")]
            {
                tokio::task::spawn_blocking(main_loop_fn);
                tokio::task::spawn_blocking(ended_loop_fn);
            }

            PlatunePlayer {
                cmd_sender: finish_tx,
                event_tx,
            }
        }

        fn clean_temp_files() {
            match std::env::temp_dir().read_dir() {
                Ok(temp_dir) => {
                    for entry in temp_dir.flatten() {
                        if entry
                            .file_name()
                            .to_string_lossy()
                            .starts_with("platunecache")
                        {
                            if let Err(e) = remove_file(entry.path()) {
                                error!("Error removing temp file {:?}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading temp dir {:?}", e);
                }
            }
        }

        pub fn subscribe(&self) -> broadcast::Receiver<PlayerEvent> {
            self.event_tx.subscribe()
        }

        pub fn set_queue(&self, queue: Vec<String>) -> Result<(), SendError> {
            self.cmd_sender
                .send(Command::SetQueue(queue))
                .map_err(|e| SendError(format!("{:?}", e)))
        }

        pub fn add_to_queue(&self, songs: Vec<String>) -> Result<(), SendError> {
            self.cmd_sender
                .send(Command::AddToQueue(songs))
                .map_err(|e| SendError(format!("{:?}", e)))
        }

        pub fn seek(&self, millis: u64) -> Result<(), SendError> {
            self.cmd_sender
                .send(Command::Seek(millis))
                .map_err(|e| SendError(format!("{:?}", e)))
        }

        pub fn start(&self) -> Result<(), SendError> {
            self.cmd_sender
                .send(Command::Start)
                .map_err(|e| SendError(format!("{:?}", e)))
        }

        pub fn stop(&self) -> Result<(), SendError> {
            self.cmd_sender
                .send(Command::Stop)
                .map_err(|e| SendError(format!("{:?}", e)))
        }

        pub fn set_volume(&self, volume: f32) -> Result<(), SendError> {
            self.cmd_sender
                .send(Command::SetVolume(volume))
                .map_err(|e| SendError(format!("{:?}", e)))
        }

        pub fn pause(&self) -> Result<(), SendError> {
            self.cmd_sender
                .send(Command::Pause)
                .map_err(|e| SendError(format!("{:?}", e)))
        }

        pub fn resume(&self) -> Result<(), SendError> {
            self.cmd_sender
                .send(Command::Resume)
                .map_err(|e| SendError(format!("{:?}", e)))
        }

        pub fn next(&self) -> Result<(), SendError> {
            self.cmd_sender
                .send(Command::Next)
                .map_err(|e| SendError(format!("{:?}", e)))
        }

        pub fn previous(&self) -> Result<(), SendError> {
            self.cmd_sender
                .send(Command::Previous)
                .map_err(|e| SendError(format!("{:?}", e)))
        }

        pub fn join(&self) -> Result<(), SendError> {
            self.cmd_sender
                .send(Command::Shutdown)
                .map_err(|e| SendError(format!("{:?}", e)))
        }
    }

    impl Drop for PlatunePlayer {
        fn drop(&mut self) {
            if let Err(e) = self.cmd_sender.send(Command::Shutdown) {
                // Receiver may already be terminated so this may not be an error
                warn!("Unable to send shutdown command {:?}", e);
            }
        }
    }
}
