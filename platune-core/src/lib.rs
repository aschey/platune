mod dto;
mod event_loop;
mod http_stream_reader;
mod output;
mod player;
mod source;
mod timer;
pub mod platune_player {
    use std::sync::mpsc::SyncSender;
    use std::{sync::mpsc, thread};
    use tokio::sync::broadcast;
    use tracing::{error, warn};

    pub use crate::dto::audio_status::AudioStatus;
    pub use crate::dto::player_event::PlayerEvent;
    pub use crate::dto::player_state::PlayerState;
    pub use crate::dto::player_status::PlayerStatus;
    use crate::event_loop::decode_loop;
    use crate::{dto::command::Command, event_loop::main_loop};
    use std::fs::remove_file;

    #[derive(Debug, Clone)]
    pub struct PlayerError(String);

    #[derive(Debug)]
    pub struct PlatunePlayer {
        cmd_sender: SyncSender<Command>,
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
            let (cmd_tx, cmd_rx) = std::sync::mpsc::sync_channel(32);
            let cmd_tx_ = cmd_tx.clone();
            let (queue_tx, queue_rx) = std::sync::mpsc::channel();
            let (decoder_tx, decoder_rx) = std::sync::mpsc::channel();

            let event_tx_ = event_tx.clone();
            let main_loop_fn = || main_loop(cmd_rx, event_tx_, queue_tx, decoder_tx);
            //let ended_loop_fn = || ended_loop(rx, finish_tx_);
            let decoder_fn = || decode_loop(queue_rx, decoder_rx, cmd_tx_);
            thread::spawn(main_loop_fn);
            //thread::spawn(ended_loop_fn);
            thread::spawn(decoder_fn);

            PlatunePlayer {
                cmd_sender: cmd_tx,
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

        pub fn set_queue(&self, queue: Vec<String>) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::SetQueue(queue))
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub fn add_to_queue(&self, songs: Vec<String>) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::AddToQueue(songs))
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub fn seek(&self, millis: u64) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Seek(millis))
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub fn get_current_status(&self) -> Result<PlayerStatus, PlayerError> {
            let (current_status_tx, current_status_rx) = mpsc::channel();
            match self
                .cmd_sender
                .send(Command::GetCurrentStatus(current_status_tx))
            {
                Ok(()) => match current_status_rx.recv() {
                    Ok(current_status) => Ok(current_status),
                    Err(e) => Err(PlayerError(format!("{:?}", e))),
                },
                Err(e) => Err(PlayerError(format!("{:?}", e))),
            }
        }

        pub fn stop(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Stop)
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub fn set_volume(&self, volume: f32) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::SetVolume(volume))
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub fn pause(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Pause)
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub fn resume(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Resume)
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub fn next(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Next)
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub fn previous(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Previous)
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub fn join(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Shutdown)
                .map_err(|e| PlayerError(format!("{:?}", e)))
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
