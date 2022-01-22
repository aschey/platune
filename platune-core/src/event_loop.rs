use crate::{
    audio_manager::AudioManager,
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse, queue_source::QueueSource,
    },
    output::CpalAudioOutput,
    player::Player,
    TwoWayReceiver, TwoWayReceiverAsync, TwoWaySenderAsync,
};
use crossbeam_channel::{Receiver, TryRecvError};
use tracing::{error, info};

pub(crate) fn decode_loop(
    queue_rx: Receiver<QueueSource>,
    volume: f64,
    mut cmd_rx: TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: TwoWaySenderAsync<Command, PlayerResponse>,
) {
    let output = CpalAudioOutput::new_output().unwrap();
    let mut audio_manager = AudioManager::new(output, volume);

    loop {
        match queue_rx.try_recv() {
            Ok(QueueSource {
                source,
                settings,
                force_restart_output,
            }) => {
                if force_restart_output {
                    audio_manager.stop();
                    audio_manager.reset(settings.resample_chunk_size);
                } else {
                    audio_manager.start();
                }
                audio_manager.decode_source(source, &mut cmd_rx, &player_cmd_tx, settings);
            }
            Err(TryRecvError::Empty) => {
                // If no pending source, stop the output to preserve cpu
                audio_manager.play_remaining();
                audio_manager.stop();
                match queue_rx.recv() {
                    Ok(QueueSource {
                        source, settings, ..
                    }) => {
                        audio_manager.reset(settings.resample_chunk_size);
                        audio_manager.decode_source(source, &mut cmd_rx, &player_cmd_tx, settings);
                    }
                    Err(_) => break,
                };
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
        }
    }
}

pub(crate) async fn main_loop(
    mut receiver: TwoWayReceiverAsync<Command, PlayerResponse>,
    mut player: Player,
) {
    while let Some(next_command) = receiver.recv().await {
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
                player.on_ended();
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
                    error!("Error sending player status");
                }
            }
            Command::Shutdown => {
                return;
            }
        }
        info!("Completed command");
    }
    info!("Request loop completed");
}
