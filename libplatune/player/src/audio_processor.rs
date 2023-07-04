use crate::{
    decoder::{Decoder, DecoderParams},
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse, processor_error::ProcessorError,
    },
    platune_player::PlayerEvent,
    two_way_channel::{TwoWayReceiver, TwoWaySender},
};
use flume::TryRecvError;
use std::time::Duration;
use tap::TapFallible;
use tracing::{error, info};

pub(crate) struct AudioProcessor<'a> {
    cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: &'a TwoWaySender<Command, PlayerResponse>,
    decoder: Decoder,
    last_send_time: Duration,
    event_tx: &'a tokio::sync::broadcast::Sender<PlayerEvent>,
}

enum InputResult {
    Continue,
    Stop,
}

impl<'a> AudioProcessor<'a> {
    pub(crate) fn new(
        decoder_params: DecoderParams,
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

        match Decoder::new(decoder_params) {
            Ok(decoder) => {
                cmd_rx
                    .respond(DecoderResponse::InitializationSucceeded)
                    .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                    .tap_err(|e| error!("Error sending decoder initialization succeeded: {e:?}"))?;

                Ok(Self {
                    decoder,
                    cmd_rx,
                    player_cmd_tx,
                    event_tx,
                    last_send_time: Duration::default(),
                })
            }
            Err(e) => {
                cmd_rx
                    .respond(DecoderResponse::InitializationFailed)
                    .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                    .tap_err(|e| error!("Error sending decoder initialization failed: {e:?}"))?;

                Err(ProcessorError::DecoderError(e))
            }
        }
    }

    pub(crate) fn sample_rate(&self) -> usize {
        self.decoder.sample_rate()
    }

    pub(crate) fn volume(&self) -> f64 {
        self.decoder.volume()
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
                        .send(PlayerEvent::Position {
                            current_position: position.clone(),
                        })
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

    pub(crate) fn current(&self) -> &[f64] {
        self.decoder.current()
    }

    pub(crate) fn next(&mut self) -> Result<Option<&[f64]>, ProcessorError> {
        match self.process_input() {
            Ok(InputResult::Continue) => {}
            Ok(InputResult::Stop) => return Ok(None),
            Err(e) => return Err(e),
        };
        match self.decoder.next() {
            Ok(Some(val)) => Ok(Some(val)),
            val => {
                self.player_cmd_tx
                    .send(Command::Ended)
                    .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                    .tap_err(|e| error!("Unable to send ended command: {e:?}"))?;
                val.map_err(ProcessorError::DecoderError)
            }
        }
    }
}
