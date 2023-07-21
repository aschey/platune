mod audio_processor;
mod dto;
mod event_loop;
mod http_stream_reader;
mod player;
mod settings;
mod two_way_channel;
pub use decal::output::{AudioBackend, CpalOutput, MockOutput};

#[cfg(feature = "ffi")]
uniffi::include_scaffolding!("player");

pub mod platune_player {
    pub use crate::dto::audio_status::AudioStatus;
    use crate::dto::decoder_command::DecoderCommand;
    use crate::dto::decoder_response::DecoderResponse;
    pub use crate::dto::player_event::PlayerEvent;
    use crate::dto::player_response::PlayerResponse;
    pub use crate::dto::player_state::PlayerState;
    use crate::dto::player_status::CurrentPosition;
    pub use crate::dto::player_status::PlayerStatus;
    use crate::event_loop::decode_loop;
    use crate::player::Player;
    pub use crate::settings::Settings;
    use crate::two_way_channel::{two_way_channel, TwoWaySender};
    use crate::{dto::command::Command, event_loop::main_loop};
    use decal::output::{AudioBackend, CpalOutput, DeviceTrait, HostTrait};
    use derivative::Derivative;
    use std::fs::remove_file;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use tap::TapFallible;
    use thiserror::Error;
    use tokio::runtime::Runtime;
    use tokio::sync::{broadcast, Mutex};
    use tracing::{error, info, warn};

    #[derive(Debug, Clone, Error)]
    #[cfg_attr(feature = "ffi", derive(uniffi::Error))]
    #[cfg_attr(feature = "ffi", uniffi(flat_error))]
    pub enum PlayerError {
        #[error("{0}")]
        Failure(String),
    }

    pub struct EventSubscription {
        rx: broadcast::Receiver<PlayerEvent>,
    }

    impl EventSubscription {
        pub async fn recv(&mut self) -> Result<PlayerEvent, PlayerError> {
            self.rx
                .recv()
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }
    }

    #[cfg(feature = "ffi")]
    #[derive(uniffi::Object)]
    pub struct FfiEventSubscription(Arc<Mutex<EventSubscription>>);

    #[cfg(feature = "ffi")]
    #[uniffi::export(async_runtime = "tokio")]
    impl FfiEventSubscription {
        pub async fn recv(&self) -> Result<PlayerEvent, PlayerError> {
            self.0
                .lock()
                .await
                .recv()
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }
    }

    #[cfg(feature = "ffi")]
    #[derive(uniffi::Object)]
    pub struct FfiPlatunePlayer {
        player: PlatunePlayer<CpalOutput>,
        _rt: Runtime,
    }

    #[cfg(feature = "ffi")]
    #[uniffi::export]
    impl FfiPlatunePlayer {
        #[cfg(feature = "ffi")]
        #[uniffi::constructor]
        pub fn new(settings: Settings) -> Arc<Self> {
            let rt = Runtime::new().unwrap();
            let _guard = rt.enter();
            Arc::new(Self {
                player: PlatunePlayer::new(CpalOutput::default(), settings),
                _rt: rt,
            })
        }

        pub fn subscribe(&self) -> Arc<FfiEventSubscription> {
            Arc::new(FfiEventSubscription(Arc::new(Mutex::new(
                self.player.subscribe(),
            ))))
        }

        pub fn output_devices(&self) -> Result<Vec<String>, PlayerError> {
            self.player.output_devices()
        }
    }

    #[cfg(feature = "ffi")]
    #[uniffi::export(async_runtime = "tokio")]
    impl FfiPlatunePlayer {
        pub async fn set_queue(&self, queue: Vec<String>) -> Result<(), PlayerError> {
            self.player.set_queue(queue).await
        }

        pub async fn add_to_queue(&self, songs: Vec<String>) -> Result<(), PlayerError> {
            self.player.add_to_queue(songs).await
        }

        pub async fn seek(&self, time: Duration) -> Result<(), PlayerError> {
            self.player.seek(time).await
        }

        pub async fn get_current_status(&self) -> Result<PlayerStatus, PlayerError> {
            self.player.get_current_status().await
        }

        pub async fn stop(&self) -> Result<(), PlayerError> {
            self.player.stop().await
        }

        pub async fn set_volume(&self, volume: f32) -> Result<(), PlayerError> {
            self.player.set_volume(volume).await
        }

        pub async fn pause(&self) -> Result<(), PlayerError> {
            self.player.pause().await
        }

        pub async fn resume(&self) -> Result<(), PlayerError> {
            self.player.resume().await
        }

        pub async fn next(&self) -> Result<(), PlayerError> {
            self.player.next().await
        }

        pub async fn previous(&self) -> Result<(), PlayerError> {
            self.player.previous().await
        }

        pub async fn set_output_device(&self, device: Option<String>) -> Result<(), PlayerError> {
            self.player.set_output_device(device).await
        }

        pub async fn join(&self) -> Result<(), PlayerError> {
            self.player.join().await
        }
    }

    #[derive(Derivative)]
    #[derivative(Debug)]
    pub struct PlatunePlayer<B>
    where
        B: AudioBackend,
    {
        cmd_sender: TwoWaySender<Command, PlayerResponse>,
        decoder_tx: TwoWaySender<DecoderCommand, DecoderResponse>,
        event_tx: broadcast::Sender<PlayerEvent>,
        decoder_handle: Option<std::thread::JoinHandle<()>>,
        #[derivative(Debug = "ignore")]
        audio_backend: B,
        joined: AtomicBool,
    }

    impl<B: AudioBackend + Send + 'static> PlatunePlayer<B> {
        pub fn new(audio_backend: B, settings: Settings) -> Self {
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
                let player =
                    Player::new(event_tx_, queue_tx, queue_rx, decoder_tx_, settings, None);
                main_loop(cmd_rx, player).await
            };
            let audio_backend_ = audio_backend.clone();
            let decoder_fn = || {
                decode_loop(
                    queue_rx_,
                    1.0,
                    decoder_rx,
                    cmd_tx_,
                    event_tx__,
                    audio_backend_,
                );
            };

            tokio::spawn(main_loop_fn);
            let decoder_handle = Some(thread::spawn(decoder_fn));

            Self {
                cmd_sender: cmd_tx,
                event_tx,
                decoder_tx,
                decoder_handle,
                audio_backend,
                joined: AtomicBool::new(false),
            }
        }

        pub fn subscribe(&self) -> EventSubscription {
            EventSubscription {
                rx: self.event_tx.subscribe(),
            }
        }

        pub fn output_devices(&self) -> Result<Vec<String>, PlayerError> {
            let devices = self
                .audio_backend
                .default_host()
                .output_devices()
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))?;

            Ok(devices
                .into_iter()
                .filter_map(|d| d.name().map(|n| n.trim_end().to_owned()).ok())
                .collect())
        }

        pub async fn set_output_device(&self, device: Option<String>) -> Result<(), PlayerError> {
            self.cmd_sender
                .send_async(Command::SetDeviceName(device))
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))
        }

        pub async fn join(&self) -> Result<(), PlayerError> {
            info!("Joining player instance");
            self.cmd_sender
                .send_async(Command::Stop)
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))?;
            info!("Sent stop command");
            self.cmd_sender
                .send_async(Command::Shutdown)
                .await
                .map_err(|e| PlayerError::Failure(format!("{e:?}")))?;
            info!("Sent shutdown command");
            self.joined.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    // #[cfg_attr(feature = "ffi", uniffi::export(async_runtime = "tokio"))]
    impl<B: AudioBackend> PlatunePlayer<B> {
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
                                current_position: Some(CurrentPosition {
                                    position: current_position.position,
                                    retrieval_time: current_position.retrieval_time,
                                }),
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

        pub async fn set_volume(&self, volume: f32) -> Result<(), PlayerError> {
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

    impl<B: AudioBackend> Drop for PlatunePlayer<B> {
        fn drop(&mut self) {
            if self.joined.load(Ordering::SeqCst) {
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
