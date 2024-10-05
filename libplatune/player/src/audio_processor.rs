use std::time::Duration;

use decal::AudioManager;
use decal::decoder::{Decoder, DecoderResult};
use decal::output::AudioBackend;
use flume::TryRecvError;
use tap::TapFallible;
use tracing::{error, info};

use crate::dto::decoder_command::DecoderCommand;
use crate::dto::decoder_response::DecoderResponse;
use crate::dto::processor_error::ProcessorError;
use crate::platune_player::PlayerEvent;
use crate::two_way_channel::TwoWayReceiver;

pub(crate) struct AudioProcessor<'a, B: AudioBackend> {
    cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
    manager: &'a mut AudioManager<f32, B>,
    decoder: Decoder<f32>,
    last_send_time: Duration,
    event_tx: &'a tokio::sync::broadcast::Sender<PlayerEvent>,
}

pub(crate) enum InputResult {
    Continue,
    Stop,
}

impl<'a, B: AudioBackend> AudioProcessor<'a, B> {
    pub(crate) fn new(
        manager: &'a mut AudioManager<f32, B>,
        decoder: Decoder<f32>,
        cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
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
                // if position.position < last_send_time, we just seeked backwards
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

    pub(crate) fn next(&mut self) -> Result<(InputResult, DecoderResult), ProcessorError> {
        match self.process_input() {
            Ok(InputResult::Continue) => {}
            Ok(InputResult::Stop) => return Ok((InputResult::Stop, DecoderResult::Finished)),
            Err(e) => return Err(e),
        };
        let res = self
            .manager
            .write(&mut self.decoder)
            .map_err(ProcessorError::WriteOutputError)?;
        Ok((InputResult::Continue, res))
    }
}
