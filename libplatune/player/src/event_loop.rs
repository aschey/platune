use std::time::Duration;

use crate::{
    audio_manager::AudioManager,
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse, queue_source::QueueSource,
    },
    output::CpalAudioOutput,
    platune_player::PlayerEvent,
    player::Player,
    two_way_channel::{TwoWayReceiver, TwoWaySender},
};

use flume::{Receiver, TryRecvError};
use tracing::{error, info};

pub(crate) fn decode_loop(
    queue_rx: Receiver<QueueSource>,
    volume: f64,
    mut cmd_rx: TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: TwoWaySender<Command, PlayerResponse>,
    event_tx: tokio::sync::broadcast::Sender<PlayerEvent>,
) {
    let output = match CpalAudioOutput::new_output(player_cmd_tx.clone()) {
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
            Ok(QueueSource {
                source,
                settings,
                force_restart_output,
            }) => {
                if force_restart_output {
                    info!("Restarting output stream");
                    audio_manager.stop();
                    if let Err(e) = audio_manager.reset(settings.resample_chunk_size) {
                        error!("Error resetting output stream: {e:?}");
                        return;
                    }
                    prev_stop_position = audio_manager.decode_source(
                        source,
                        &mut cmd_rx,
                        &player_cmd_tx,
                        &event_tx,
                        settings,
                        Some(prev_stop_position),
                    );
                } else {
                    if let Err(e) = audio_manager.start() {
                        error!("Error starting output stream: {e:?}");
                    }
                    prev_stop_position = audio_manager.decode_source(
                        source,
                        &mut cmd_rx,
                        &player_cmd_tx,
                        &event_tx,
                        settings,
                        None,
                    );
                }
            }
            Err(TryRecvError::Empty) => {
                // If no pending source, stop the output to preserve cpu
                audio_manager.play_remaining();
                audio_manager.stop();
                match queue_rx.recv() {
                    Ok(QueueSource {
                        source, settings, ..
                    }) => {
                        if let Err(e) = audio_manager.reset(settings.resample_chunk_size) {
                            error!("Error resetting output stream: {e:?}");
                            return;
                        }
                        prev_stop_position = audio_manager.decode_source(
                            source,
                            &mut cmd_rx,
                            &player_cmd_tx,
                            &event_tx,
                            settings,
                            None,
                        );
                    }
                    Err(_) => break,
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
) {
    while let Ok(next_command) = receiver.recv_async().await {
        info!("Got command {:?}", next_command);
        match next_command {
            Command::SetQueue(songs) => {
                player.set_queue(songs).await;
            }
            Command::AddToQueue(song) => {
                player.add_to_queue(song).await;
            }
            Command::Seek(millis) => {
                player.seek(millis).await;
            }
            Command::SetVolume(volume) => {
                player.set_volume(volume).await;
            }
            Command::Pause => {
                player.pause().await;
            }
            Command::Resume => {
                player.play().await;
            }
            Command::Stop => {
                player.stop().await;
            }
            Command::Ended => {
                player.on_ended().await;
            }
            Command::Next => {
                player.go_next().await;
            }
            Command::Previous => {
                player.go_previous().await;
            }
            Command::GetCurrentStatus => {
                let current_status = player.get_current_status();
                if let Err(e) = receiver.respond(PlayerResponse::StatusResponse(current_status)) {
                    error!("Error sending player status: {e:?}");
                }
            }
            Command::Reset => {
                player.reset().await;
            }
            Command::Shutdown => {
                return;
            }
        }
        info!("Completed command");
    }
    info!("Request loop completed");
}
