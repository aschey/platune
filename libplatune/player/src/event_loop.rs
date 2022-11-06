use std::time::Duration;

use crate::{
    audio_manager::AudioManager,
    dto::{
        command::Command,
        decoder_command::DecoderCommand,
        decoder_response::DecoderResponse,
        player_response::PlayerResponse,
        queue_source::{QueueSource, QueueStartMode},
    },
    output::CpalAudioOutput,
    platune_player::PlayerEvent,
    player::Player,
    two_way_channel::{TwoWayReceiver, TwoWaySender},
};

use crate::audio_output::*;
use flume::{Receiver, TryRecvError};
use tap::TapFallible;
use tokio_graceful_shutdown::{FutureExt, SubsystemHandle};
use tracing::{error, info};

pub(crate) fn decode_loop(
    queue_rx: Receiver<QueueSource>,
    volume: f64,
    mut cmd_rx: TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: TwoWaySender<Command, PlayerResponse>,
    event_tx: tokio::sync::broadcast::Sender<PlayerEvent>,
    host: Host,
) {
    let output = match CpalAudioOutput::new_output(host, player_cmd_tx.clone()) {
        Ok(output) => output,
        Err(e) => {
            error!("Error opening audio output: {e:?}");
            return;
        }
    };
    let mut audio_manager = AudioManager::new(output, volume);
    let mut prev_stop_position = Duration::default();
    loop {
        match queue_rx.try_recv() {
            Ok(queue_source) => {
                info!("try_recv got source {queue_source:?}");
                match queue_source.queue_start_mode {
                    QueueStartMode::ForceRestart => {
                        info!("Restarting output stream");
                        audio_manager.stop();
                        if audio_manager
                            .reset(queue_source.settings.resample_chunk_size)
                            .tap_err(|e| error!("Error resetting output stream: {e:?}"))
                            .is_err()
                        {
                            return;
                        }

                        prev_stop_position = audio_manager.decode_source(
                            queue_source,
                            &mut cmd_rx,
                            &player_cmd_tx,
                            &event_tx,
                            Some(prev_stop_position),
                        );
                    }
                    QueueStartMode::Normal => {
                        if audio_manager
                            .start()
                            .tap_err(|e| error!("Error starting output stream: {e:?}"))
                            .is_err()
                        {
                            return;
                        }

                        prev_stop_position = audio_manager.decode_source(
                            queue_source,
                            &mut cmd_rx,
                            &player_cmd_tx,
                            &event_tx,
                            None,
                        );
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
                        if let Err(e) =
                            audio_manager.reset(queue_source.settings.resample_chunk_size)
                        {
                            error!("Error resetting output stream: {e:?}");
                            return;
                        }
                        prev_stop_position = audio_manager.decode_source(
                            queue_source,
                            &mut cmd_rx,
                            &player_cmd_tx,
                            &event_tx,
                            None,
                        );
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

pub(crate) async fn main_loop(
    mut receiver: TwoWayReceiver<Command, PlayerResponse>,
    mut player: Player,
    subsys: SubsystemHandle,
) -> Result<(), String> {
    info!("Starting request loop");
    while let Ok(Ok(next_command)) = receiver.recv_async().cancel_on_shutdown(&subsys).await {
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
            Command::Reset => {
                player.reset().await?;
            }
        }
        info!("Completed command");
    }
    player.stop().await?;
    info!("Request loop completed");
    Ok(())
}
