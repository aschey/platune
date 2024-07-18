use std::time::Duration;

use decal::decoder::{
    Decoder, DecoderError, DecoderResult, DecoderSettings, ResamplerSettings, Source,
};
use decal::output::{AudioBackend, OutputBuilder, OutputSettings, WriteBlockingError};
use decal::{AudioManager, WriteOutputError};
use flume::{Receiver, TryRecvError};
use tap::TapFallible;
use tracing::{error, info, warn};

use crate::audio_processor::{AudioProcessor, InputResult};
use crate::dto::command::Command;
use crate::dto::decoder_command::DecoderCommand;
use crate::dto::decoder_response::DecoderResponse;
use crate::dto::player_response::PlayerResponse;
use crate::dto::processor_error::ProcessorError;
use crate::dto::queue_source::{QueueSource, QueueStartMode};
use crate::platune_player::PlayerEvent;
use crate::player::Player;
use crate::two_way_channel::{TwoWayReceiver, TwoWaySender};

pub(crate) fn decode_loop<B: AudioBackend>(
    queue_rx: Receiver<QueueSource>,
    volume: f32,
    mut cmd_rx: TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: TwoWaySender<Command, PlayerResponse>,
    event_tx: tokio::sync::broadcast::Sender<PlayerEvent>,
    audio_backend: B,
) {
    let player_cmd_tx_ = player_cmd_tx.clone();
    let output_builder = OutputBuilder::new(
        audio_backend,
        OutputSettings::default(),
        move || {
            player_cmd_tx_
                .send(Command::Reset)
                .tap_err(|e| error!("Error sending reset command: {e}"))
                .ok();
        },
        |err| error!("Output error: {err}"),
    );
    let mut manager = loop {
        if let Ok(manager) =
            AudioManager::<f32, _>::new(output_builder.clone(), ResamplerSettings::default())
                .tap_err(|e| error!("Error creating audio manager: {e:?}"))
        {
            break manager;
        } else {
            std::thread::sleep(Duration::from_secs(1));
        }
    };
    manager.set_volume(volume);
    let mut last_stop_position = Duration::default();

    loop {
        let decoder = match queue_rx.try_recv() {
            Ok(queue_source) => {
                info!("Got source on initial attempt");
                init_source(&mut manager, &queue_source);

                match queue_source.queue_start_mode {
                    QueueStartMode::ForceRestart {
                        device_name,
                        paused,
                    } => {
                        if let Ok(decoder) = handle_force_restart(
                            &mut manager,
                            device_name,
                            queue_source.source,
                            last_stop_position,
                            paused,
                        )
                        .tap_err(|e| error!("Error during force restart {e:?}"))
                        {
                            decoder
                        } else {
                            continue;
                        }
                    }
                    QueueStartMode::Normal => {
                        if let Ok(decoder) = manager
                            .init_decoder(
                                queue_source.source,
                                DecoderSettings {
                                    enable_gapless: true,
                                },
                            )
                            .tap_err(|e| error!("Error initializing decoder {e:?}"))
                        {
                            decoder
                        } else {
                            continue;
                        }

                        let _ = manager
                            .initialize(&mut decoder)
                            .tap_err(|e| error!("error initializing decoder {e:?}"));
                        decoder
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                info!("No sources on initial attempt, waiting");
                let _ = manager
                    .flush()
                    .tap_err(|e| error!("Error flushing output: {e:?}"));
                match queue_rx.recv() {
                    Ok(queue_source) => {
                        info!("Got source after waiting");
                        init_source(&mut manager, &queue_source);

                        match queue_source.queue_start_mode {
                            QueueStartMode::ForceRestart {
                                device_name,
                                paused,
                            } => {
                                if let Ok(decoder) = handle_force_restart(
                                    &mut manager,
                                    device_name,
                                    queue_source.source,
                                    last_stop_position,
                                    paused,
                                )
                                .tap_err(|e| error!("Error handling force restart: {e:?}"))
                                {
                                    decoder
                                } else {
                                    continue;
                                }
                            }
                            QueueStartMode::Normal => {
                                if let Ok(mut decoder) = manager
                                    .init_decoder(
                                        queue_source.source,
                                        DecoderSettings {
                                            enable_gapless: true,
                                        },
                                    )
                                    .tap_err(|e| error!("Error initializing decoder {e:?}"))
                                {
                                    decoder
                                } else {
                                    continue;
                                }
                                manager.reset(&mut decoder).ok();
                                decoder
                            }
                        }
                    }
                    Err(_) => {
                        info!("Queue receiver disconnected");
                        return;
                    }
                }
            }
            Err(TryRecvError::Disconnected) => {
                info!("Decoder thread receiver disconnected. Terminating.");
                break;
            }
        };
        info!("Creating processor");
        if let Ok(mut processor) =
            AudioProcessor::new(&mut manager, decoder, &mut cmd_rx, &event_tx)
                .tap_err(|e| error!("Error creating processor: {e}"))
        {
            loop {
                match processor.next() {
                    Ok((InputResult::Stop, _)) => {
                        // Don't send Command::Ended when we explicitly requested to stop
                        // because we don't want to initialize the next track
                        break;
                    }
                    Ok((_, DecoderResult::Unfinished)) => {}
                    Ok((_, DecoderResult::Finished))
                    | Err(ProcessorError::WriteOutputError(
                        WriteOutputError::WriteBlockingError {
                            decoder_result: DecoderResult::Finished,
                            error: _,
                        },
                    )) => {
                        info!("Sending ended event");
                        player_cmd_tx
                            .send(Command::Ended)
                            .tap_err(|e| error!("Unable to send ended command: {e:?}"))
                            .ok();
                        break;
                    }
                    Err(ProcessorError::WriteOutputError(
                        WriteOutputError::DecoderError(DecoderError::ResetRequired)
                        | WriteOutputError::WriteBlockingError {
                            decoder_result: _,
                            error: WriteBlockingError::OutputStalled,
                        },
                    )) => {
                        player_cmd_tx
                            .send(Command::Reset)
                            .tap_err(|e| error!("Error sending reset command: {e}"))
                            .ok();
                    }
                    Err(e) => {
                        error!("Error while decoding: {e:?}");
                        break;
                    }
                }
            }
            last_stop_position = processor.current_position();
        }
    }
}

fn init_source<B: AudioBackend>(manager: &mut AudioManager<f32, B>, queue_source: &QueueSource) {
    info!("got source {queue_source:?}");
    if let Some(volume) = queue_source.volume {
        manager.set_volume(volume);
    }
    manager.set_resampler_settings(ResamplerSettings {
        chunk_size: queue_source.settings.resample_chunk_size,
    });
}

fn handle_force_restart<B: AudioBackend>(
    manager: &mut AudioManager<f32, B>,
    device_name: Option<String>,
    source: Box<dyn Source>,
    last_stop_position: Duration,
    paused: bool,
) -> Result<Decoder<f32>, DecoderError> {
    info!("Force restarting. Paused={paused}, last_stop_position={last_stop_position:?}");
    manager.set_device(device_name);
    let mut decoder = manager.init_decoder(
        source,
        DecoderSettings {
            enable_gapless: true,
        },
    )?;

    decoder
        .seek(last_stop_position)
        .map_err(|e| warn!("Error seeking: {e}"))
        .ok();

    if paused {
        decoder.pause();
    }
    manager.reset(&mut decoder).ok();
    Ok(decoder)
}

pub(crate) async fn main_loop(
    mut receiver: TwoWayReceiver<Command, PlayerResponse>,
    mut player: Player,
) -> Result<(), String> {
    // TODO send something to tell clients to clear their state on server restart
    while let Ok(next_command) = receiver.recv_async().await {
        info!("Got command {:?}", next_command);
        match next_command {
            Command::SetQueue(songs) => {
                player.set_queue(songs).await?;
            }
            Command::AddToQueue(song) => {
                player.add_to_queue(song).await?;
            }
            Command::Seek(millis) => {
                player.seek(millis).await;
            }
            Command::SetVolume(volume) => {
                player.set_volume(volume).await?;
            }
            Command::Pause => {
                player.pause().await?;
            }
            Command::Resume => {
                player.play().await?;
            }
            Command::Stop => {
                player.stop().await?;
            }
            Command::Ended => {
                player.on_ended().await;
            }
            Command::Next => {
                player.go_next().await?;
            }
            Command::Previous => {
                player.go_previous().await?;
            }
            Command::GetCurrentStatus => {
                let current_status = player.get_current_status();
                if let Err(e) = receiver.respond(PlayerResponse::StatusResponse(current_status)) {
                    error!("Error sending player status: {e:?}");
                }
            }
            Command::SetDeviceName(name) => {
                player.set_device_name(name).await?;
            }
            Command::Reset => {
                player.reset().await?;
            }
            Command::Shutdown => {
                return Ok(());
            }
        }
        info!("Completed command");
    }
    info!("Request loop completed");
    Ok(())
}
