use crate::{
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse, processor_error::ProcessorError,
    },
    platune_player::PlayerEvent,
    two_way_channel::{TwoWayReceiver, TwoWaySender},
};
use decal::{
    decoder::{Decoder, DecoderResult},
    output::AudioBackend,
    AudioManager, WriteOutputError,
};
use flume::TryRecvError;
use std::time::Duration;
use tap::TapFallible;
use tracing::{error, info};

pub(crate) struct AudioProcessor<'a, B: AudioBackend> {
    cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: &'a TwoWaySender<Command, PlayerResponse>,
    manager: &'a mut AudioManager<f32, B>,
    decoder: Decoder<f32>,
    last_send_time: Duration,
    event_tx: &'a tokio::sync::broadcast::Sender<PlayerEvent>,
}

enum InputResult {
    Continue,
    Stop,
}

impl<'a, B: AudioBackend> AudioProcessor<'a, B> {
    pub(crate) fn new(
        manager: &'a mut AudioManager<f32, B>,
        decoder: Decoder<f32>,
        cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
        player_cmd_tx: &'a TwoWaySender<Command, PlayerResponse>,
        event_tx: &'a tokio::sync::broadcast::Sender<PlayerEvent>,
    ) -> Result<Self, ProcessorError> {
        match cmd_rx.recv() {
            Ok(DecoderCommand::WaitForInitialization) => {
                info!("Notifying decoder started");
            }
            Ok(cmd) => {
                error!("Got unexpected command {cmd:?}");
            }
            Err(e) => {
                error!("Error receiving initialization message {e:?}");
            }
        }

        cmd_rx
            .respond(DecoderResponse::InitializationSucceeded)
            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
            .tap_err(|e| error!("Error sending decoder initialization succeeded: {e:?}"))?;

        Ok(Self {
            decoder,
            manager,
            cmd_rx,
            player_cmd_tx,
            event_tx,
            last_send_time: Duration::default(),
        })
    }

    pub(crate) fn current_position(&self) -> Duration {
        self.decoder.current_position().position
    }

    fn process_input(&mut self) -> Result<InputResult, ProcessorError> {
        match self.cmd_rx.try_recv() {
            Ok(command) => {
                info!("Got decoder command {:?}", command);

                match command {
                    DecoderCommand::Play => {
                        self.decoder.resume();

                        self.cmd_rx
                            .respond(DecoderResponse::Received)
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| error!("Error sending stopped response: {e:?}"))?;
                    }
                    DecoderCommand::Stop => {
                        info!("Completed decoder command");

                        self.cmd_rx
                            .respond(DecoderResponse::Received)
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| error!("Error sending stopped response: {e:?}"))?;

                        return Ok(InputResult::Stop);
                    }
                    DecoderCommand::Seek(time) => {
                        let seek_response = match self.decoder.seek(time) {
                            Ok(seeked_to) => Ok(seeked_to.actual_ts),
                            Err(e) => Err(e.to_string()),
                        };

                        self.cmd_rx
                            .respond(DecoderResponse::SeekResponse(seek_response))
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| error!("Unable to send seek result: {e:?}"))?;
                    }
                    DecoderCommand::Pause => {
                        self.decoder.pause();

                        self.cmd_rx
                            .respond(DecoderResponse::Received)
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| error!("Error sending stopped response: {e:?}"))?;
                    }
                    DecoderCommand::SetVolume(volume) => {
                        self.manager.set_volume(volume);
                        self.decoder.set_volume(volume);

                        self.cmd_rx
                            .respond(DecoderResponse::Received)
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| error!("Error sending set volume response: {e:?}"))?;
                    }
                    DecoderCommand::GetCurrentPosition => {
                        let time = self.decoder.current_position();

                        self.cmd_rx
                            .respond(DecoderResponse::CurrentPositionResponse(time))
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| {
                                error!("Unable to send current position response: {e:?}")
                            })?;
                    }
                    DecoderCommand::WaitForInitialization => {
                        unreachable!("Should only send this during initialization");
                    }
                }
                info!("Completed decoder command");
            }
            Err(TryRecvError::Empty) => {
                let position = self.decoder.current_position();
                // if position.postition < last_send_time, we just seeked backwards
                if position.position < self.last_send_time
                    || position.position - self.last_send_time >= Duration::from_secs(10)
                {
                    self.event_tx
                        .send(PlayerEvent::Position(position.clone()))
                        .unwrap_or_default();
                    self.last_send_time = position.position;
                }
            }
            Err(TryRecvError::Disconnected) => {
                info!("Decoder command sender has disconnected");
                return Ok(InputResult::Stop);
            }
        }

        Ok(InputResult::Continue)
    }

    pub(crate) fn next(&mut self) -> Result<DecoderResult, ProcessorError> {
        match self.process_input() {
            Ok(InputResult::Continue) => {}
            Ok(InputResult::Stop) => return Ok(DecoderResult::Finished),

            Err(e) => return Err(e),
        };
        match self.manager.write(&mut self.decoder) {
            Ok(DecoderResult::Unfinished)
            | Err(WriteOutputError::WriteBlockingError {
                decoder_result: DecoderResult::Unfinished,
                error: _,
            }) => Ok(DecoderResult::Unfinished),
            val => {
                self.player_cmd_tx
                    .send(Command::Ended)
                    .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                    .tap_err(|e| error!("Unable to send ended command: {e:?}"))?;
                val.map_err(ProcessorError::WriteOutputError)
            }
        }
    }
}
