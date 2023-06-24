use std::{error::Error, sync::Arc, time::Duration};

use crate::{
    audio_manager::AudioManager,
    dto::{
        command::Command,
        decoder_command::DecoderCommand,
        decoder_response::DecoderResponse,
        player_response::PlayerResponse,
        queue_source::{QueueSource, QueueStartMode},
    },
    output::OutputBuilder,
    platune_player::PlayerEvent,
    player::Player,
    two_way_channel::{TwoWayReceiver, TwoWaySender},
};

use crate::audio_output::*;
use flume::{Receiver, TryRecvError};
use tap::TapFallible;
use tracing::{error, info};

pub(crate) fn decode_loop(
    queue_rx: Receiver<QueueSource>,
    volume: f32,
    mut cmd_rx: TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: TwoWaySender<Command, PlayerResponse>,
    event_tx: tokio::sync::broadcast::Sender<PlayerEvent>,
    host: Arc<Host>,
) {
    let output = OutputBuilder::new(host, player_cmd_tx.clone());
    let Ok(mut audio_manager) =
        AudioManager::new(output, volume).tap_err(|e| error!("Error creating audio manager: {e:?}")) else {
            return;
        };
    let mut prev_stop_position = Duration::default();

    loop {
        match queue_rx.try_recv() {
            Ok(queue_source) => {
                info!("try_recv got source {queue_source:?}");
                let start_mode = queue_source.queue_start_mode.clone();
                match start_mode {
                    QueueStartMode::ForceRestart { device_name } => {
                        if let Ok(pos) = handle_force_restart(
                            queue_source,
                            &mut audio_manager,
                            device_name,
                            prev_stop_position,
                            &mut cmd_rx,
                            &player_cmd_tx,
                            &event_tx,
                        ) {
                            prev_stop_position = pos;
                        } else {
                            return;
                        }
                    }
                    QueueStartMode::Normal => {
                        let Ok(mut processor) = audio_manager
                            .initialize_processor(
                                queue_source.source,
                                queue_source.volume,
                                &mut cmd_rx,
                                &player_cmd_tx,
                                &event_tx,
                                None,
                            ).tap_err(|e| error!("Error initializing processor: {e:?}"))
                            else {
                                return;
                            };

                        if audio_manager
                            .start()
                            .tap_err(|e| error!("Error starting output stream: {e:?}"))
                            .is_err()
                        {
                            return;
                        }

                        prev_stop_position =
                            audio_manager.decode_source(&mut processor, &queue_source.settings);
                    }
                }
            }
            Err(TryRecvError::Empty) => {
                // If no pending source, stop the output to preserve cpu
                info!("No pending source, stopping output");
                audio_manager.play_remaining();
                audio_manager.stop();
                match queue_rx.recv() {
                    Ok(queue_source) => {
                        info!("recv got source {queue_source:?}");
                        let start_mode = queue_source.queue_start_mode.clone();
                        match start_mode {
                            QueueStartMode::ForceRestart { device_name } => {
                                if let Ok(pos) = handle_force_restart(
                                    queue_source,
                                    &mut audio_manager,
                                    device_name,
                                    prev_stop_position,
                                    &mut cmd_rx,
                                    &player_cmd_tx,
                                    &event_tx,
                                ) {
                                    prev_stop_position = pos;
                                } else {
                                    return;
                                }
                            }
                            QueueStartMode::Normal => {
                                let Ok(mut processor) = audio_manager
                                    .initialize_processor(
                                        queue_source.source,
                                        queue_source.volume,
                                        &mut cmd_rx,
                                        &player_cmd_tx,
                                        &event_tx,
                                        None,
                                    )
                                    .tap_err(|e| error!("Error initializing processor: {e:?}")) else {
                                        return;
                                    };
                                if audio_manager
                                    .reset(
                                        processor.output_config(),
                                        queue_source.settings.resample_chunk_size,
                                    )
                                    .tap_err(|e| error!("Error resetting output stream: {e:?}"))
                                    .is_err()
                                {
                                    return;
                                }

                                prev_stop_position = audio_manager
                                    .decode_source(&mut processor, &queue_source.settings);
                            }
                        }
                    }
                    Err(_) => {
                        info!("Queue receiver disconnected");
                    }
                };
            }
            Err(TryRecvError::Disconnected) => {
                info!("Decoder thread receiver disconnected. Terminating.");
                break;
            }
        }
    }
}

fn handle_force_restart(
    queue_source: QueueSource,
    audio_manager: &mut AudioManager,
    device_name: Option<String>,
    prev_stop_position: Duration,
    cmd_rx: &mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: &TwoWaySender<Command, PlayerResponse>,
    event_tx: &tokio::sync::broadcast::Sender<PlayerEvent>,
) -> Result<Duration, Box<dyn Error>> {
    info!("Restarting output stream");
    audio_manager.stop();
    audio_manager.set_device_name(device_name);

    let mut processor = audio_manager
        .initialize_processor(
            queue_source.source,
            queue_source.volume,
            cmd_rx,
            player_cmd_tx,
            event_tx,
            Some(prev_stop_position),
        )
        .tap_err(|e| error!("Error initializing processor: {e:?}"))?;

    audio_manager
        .reset(
            processor.output_config(),
            queue_source.settings.resample_chunk_size,
        )
        .tap_err(|e| error!("Error resetting output stream: {e:?}"))?;

    Ok(audio_manager.decode_source(&mut processor, &queue_source.settings))
}

pub(crate) async fn main_loop(
    mut receiver: TwoWayReceiver<Command, PlayerResponse>,
    mut player: Player,
) -> Result<(), String> {
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
