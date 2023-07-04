mod audio_manager;
mod audio_output;
mod audio_processor;
mod channel_buffer;
mod decoder;
mod dto;
mod event_loop;
mod http_stream_reader;
#[cfg(test)]
mod mock_output;
mod output;
mod player;
mod settings;
mod source;
mod two_way_channel;
mod vec_ext;

// use crate::platune_player::*;
uniffi::include_scaffolding!("player");

pub mod platune_player {
    use crate::audio_output::*;
    pub use crate::dto::audio_status::AudioStatus;
    use crate::dto::decoder_command::DecoderCommand;
    use crate::dto::decoder_response::DecoderResponse;
    pub use crate::dto::player_event::PlayerEvent;
    use crate::dto::player_response::PlayerResponse;
    pub use crate::dto::player_state::PlayerState;
    pub use crate::dto::player_status::PlayerStatus;
    use crate::event_loop::decode_loop;
    use crate::player::Player;
    pub use crate::settings::Settings;
    use crate::two_way_channel::{two_way_channel, TwoWaySender};
    use crate::{dto::command::Command, event_loop::main_loop};
    use std::fs::remove_file;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use tap::TapFallible;
    use thiserror::Error;
    use tokio::sync::broadcast;
    use tracing::{error, info, warn};

    #[derive(Debug, Clone, Error, uniffi::Error)]
    #[uniffi(flat_error)]

    pub enum PlayerError {
        #[error("{0}")]
        Failure(String),
    }

    #[derive(uniffi::Object)]
    pub struct EventSubscription {
        rx: Arc<tokio::sync::Mutex<broadcast::Receiver<PlayerEvent>>>,
    }

    #[uniffi::export(async_runtime = "tokio")]
    impl EventSubscription {
        pub async fn recv(&self) -> Result<PlayerEvent, PlayerError> {
            self.rx
                .lock()
                .await
                .recv()
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }
    }

    #[derive(Debug, uniffi::Object)]
    pub struct PlatunePlayer {
        cmd_sender: TwoWaySender<Command, PlayerResponse>,
        decoder_tx: TwoWaySender<DecoderCommand, DecoderResponse>,
        event_tx: broadcast::Sender<PlayerEvent>,
        decoder_handle: Option<std::thread::JoinHandle<()>>,
        joined: bool,
    }

    impl PlatunePlayer {
        #[uniffi::constructor]
        pub fn new(settings: Settings) -> Self {
            clean_temp_files();

            let (event_tx, _) = broadcast::channel(32);
            let event_tx_ = event_tx.clone();
            let event_tx__ = event_tx.clone();
            let (cmd_tx, cmd_rx) = two_way_channel();
            let cmd_tx_ = cmd_tx.clone();
            let (queue_tx, queue_rx) = flume::bounded(2);
            let queue_rx_ = queue_rx.clone();
            let (decoder_tx, decoder_rx) = two_way_channel();
            let decoder_tx_ = decoder_tx.clone();

            let main_loop_fn = async move {
                let player = Player::new(event_tx_, queue_tx, queue_rx, decoder_tx_, settings);
                main_loop(cmd_rx, player).await
            };

            let decoder_fn = || {
                decode_loop(
                    queue_rx_,
                    1.0,
                    decoder_rx,
                    cmd_tx_,
                    event_tx__,
                    default_host(),
                );
            };

            tokio::spawn(main_loop_fn);
            let decoder_handle = Some(thread::spawn(decoder_fn));

            PlatunePlayer {
                cmd_sender: cmd_tx,
                event_tx,
                decoder_tx,
                decoder_handle,
                joined: false,
            }
        }

        pub fn subscribe(&self) -> EventSubscription {
            EventSubscription {
                rx: Arc::new(tokio::sync::Mutex::new(self.event_tx.subscribe())),
            }
        }

        pub fn join(&mut self) -> Result<(), PlayerError> {
            info!("Joining player instance");
            self.cmd_sender
                .send(Command::Stop)
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))?;
            info!("Sent stop command");
            self.cmd_sender
                .send(Command::Shutdown)
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))?;
            info!("Sent shutdown command");
            self.joined = true;
            Ok(())
        }
    }

    #[uniffi::export(async_runtime = "tokio")]
    impl PlatunePlayer {
        pub async fn set_queue(&self, queue: Vec<String>) -> Result<(), PlayerError> {
            self.cmd_sender
                .send_async(Command::SetQueue(queue))
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }

        pub async fn add_to_queue(&self, songs: Vec<String>) -> Result<(), PlayerError> {
            self.cmd_sender
                .send_async(Command::AddToQueue(songs))
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }

        pub async fn seek(&self, time: Duration) -> Result<(), PlayerError> {
            self.cmd_sender
                .send_async(Command::Seek(time))
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }

        pub async fn get_current_status(&self) -> Result<PlayerStatus, PlayerError> {
            let track_status = match self
                .cmd_sender
                .get_response(Command::GetCurrentStatus)
                .await
            {
                Ok(PlayerResponse::StatusResponse(track_status)) => track_status,
                Err(e) => return Err(PlayerError::Failure(format!("{e:?}"))),
            };

            match track_status.status {
                AudioStatus::Stopped => Ok(PlayerStatus {
                    current_position: None,
                    track_status,
                }),
                _ => {
                    match self
                        .decoder_tx
                        .get_response(DecoderCommand::GetCurrentPosition)
                        .await
                    {
                        Ok(DecoderResponse::CurrentPositionResponse(current_position)) => {
                            Ok(PlayerStatus {
                                current_position: Some(current_position),
                                track_status,
                            })
                        }
                        Err(e) => Err(PlayerError::Failure(format!("{e:?}"))),
                        _ => unreachable!("Should only receive CurrentPositionResponse"),
                    }
                }
            }
        }

        pub async fn stop(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send_async(Command::Stop)
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }

        pub async fn set_volume(&self, volume: f64) -> Result<(), PlayerError> {
            self.cmd_sender
                .send_async(Command::SetVolume(volume))
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }

        pub async fn pause(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send_async(Command::Pause)
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }

        pub async fn resume(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send_async(Command::Resume)
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }

        pub async fn next(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send_async(Command::Next)
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }

        pub async fn previous(&self) -> Result<(), PlayerError> {
            self.cmd_sender
                .send_async(Command::Previous)
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }
    }

    impl Drop for PlatunePlayer {
        fn drop(&mut self) {
            if self.joined {
                info!("Waiting for decoder thread to terminate");
                let _ = self
                    .decoder_handle
                    .take()
                    .expect("decoder_handle should not be None")
                    .join()
                    .tap_err(|e| warn!("Error terminating decoder thread: {:?}", e));

                info!("Decoder thread terminated");
            } else {
                info!("join() not called, won't wait for decoder thread to terminate");
            }
        }
    }

    fn clean_temp_files() {
        if let Ok(temp_dir) = std::env::temp_dir()
            .read_dir()
            .tap_err(|e| error!("Error reading temp dir {:?}", e))
        {
            for entry in temp_dir.flatten() {
                if entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("platunecache")
                {
                    let _ = remove_file(entry.path())
                        .tap_err(|e| error!("Error removing temp file {:?}", e));
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;
