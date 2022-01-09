mod dto;
mod event_loop;
mod http_stream_reader;
mod output;
mod player;
mod source;
mod timer;
pub mod platune_player {
    use crossbeam_channel::{bounded, unbounded, Sender};
    use std::thread;
    use tokio::sync::broadcast;
    use tracing::{error, info, warn};

    pub use crate::dto::audio_status::AudioStatus;
    pub use crate::dto::player_event::PlayerEvent;
    pub use crate::dto::player_state::PlayerState;
    pub use crate::dto::player_status::PlayerStatus;
    use crate::event_loop::{decode_loop, CurrentTime, DecoderCommand};
    use crate::{dto::command::Command, event_loop::main_loop};
    use std::fs::remove_file;

    #[derive(Debug, Clone)]
    pub struct PlayerError(String);

    #[derive(Debug)]
    pub struct PlatunePlayer {
        cmd_sender: tokio::sync::mpsc::Sender<Command>,
        decoder_tx: Sender<DecoderCommand>,
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
            let event_tx_ = event_tx.clone();
            let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(32);
            let cmd_tx_ = cmd_tx.clone();
            let (queue_tx, queue_rx) = crossbeam_channel::bounded(2);
            let queue_rx_ = queue_rx.clone();
            let (decoder_tx, decoder_rx) = unbounded();
            let decoder_tx_ = decoder_tx.clone();

            let main_loop_fn =
                async move { main_loop(cmd_rx, event_tx_, queue_tx, queue_rx, decoder_tx_).await };
            let decoder_fn = || decode_loop(queue_rx_, decoder_rx, cmd_tx_);

            tokio::spawn(main_loop_fn);
            thread::spawn(decoder_fn);

            PlatunePlayer {
                cmd_sender: cmd_tx,
                event_tx,
                decoder_tx,
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

        pub async fn set_queue(&self, queue: Vec<String>) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::SetQueue(queue))
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn add_to_queue(&self, songs: Vec<String>) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::AddToQueue(songs))
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn seek(&self, millis: u64) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Seek(millis))
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn get_current_status(&self) -> Result<PlayerStatus, PlayerError> {
            let (current_status_tx, current_status_rx) = tokio::sync::oneshot::channel();

            let track_status = match self
                .cmd_sender
                .send(Command::GetCurrentStatus(current_status_tx))
                .await
            {
                Ok(()) => match current_status_rx.await {
                    Ok(current_status) => current_status,
                    Err(e) => return Err(PlayerError(format!("{:?}", e))),
                },
                Err(e) => return Err(PlayerError(format!("{:?}", e))),
            };

            match track_status.status {
                AudioStatus::Stopped => Ok(PlayerStatus {
                    current_time: CurrentTime {
                        current_time: None,
                        retrieval_time: None,
                    },
                    track_status,
                }),
                _ => {
                    let (decoder_time_tx, decoder_time_rx) = tokio::sync::oneshot::channel();
                    self.decoder_tx
                        .send(DecoderCommand::GetCurrentTime(decoder_time_tx))
                        .unwrap();
                    let current_time = decoder_time_rx.await.unwrap();

                    Ok(PlayerStatus {
                        current_time,
                        track_status,
                    })
                }
            }
        }

        pub async fn stop(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Stop)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn set_volume(&self, volume: f32) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::SetVolume(volume))
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn pause(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Pause)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn resume(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Resume)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn next(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Next)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn previous(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Previous)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }

        pub async fn join(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send(Command::Shutdown)
                .await
                .map_err(|e| PlayerError(format!("{:?}", e)))
        }
    }

    impl Drop for PlatunePlayer {
        fn drop(&mut self) {
            if let Err(e) = self.cmd_sender.try_send(Command::Shutdown) {
                // Receiver may already be terminated so this may not be an error
                warn!("Unable to send shutdown command {:?}", e);
            }
        }
    }
}
