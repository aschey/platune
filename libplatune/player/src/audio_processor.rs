use crate::{
    decoder::{Decoder, DecoderError, DecoderParams},
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse,
    },
    platune_player::PlayerEvent,
    two_way_channel::{TwoWayReceiver, TwoWaySender},
};
use flume::TryRecvError;
use std::time::Duration;
use tracing::{error, info};

pub(crate) struct AudioProcessor<'a> {
    cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: &'a TwoWaySender<Command, PlayerResponse>,
    decoder: Decoder,
    last_send_time: Duration,
    event_tx: &'a tokio::sync::broadcast::Sender<PlayerEvent>,
}

impl<'a> AudioProcessor<'a> {
    pub(crate) fn new(
        decoder_params: DecoderParams,
        cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
        player_cmd_tx: &'a TwoWaySender<Command, PlayerResponse>,
        event_tx: &'a tokio::sync::broadcast::Sender<PlayerEvent>,
    ) -> Result<Self, DecoderError> {
        let decoder = Decoder::new(decoder_params)?;

        match cmd_rx.recv() {
            Ok(DecoderCommand::WaitForInitialization) => {
                info!("Notifying decoder started");
                if let Err(e) = cmd_rx.respond(DecoderResponse::Received) {
                    error!("Error sending decoder started response {e:?}");
                }
            }
            Ok(cmd) => {
                error!("Got unexpected command {cmd:?}");
            }
            Err(e) => {
                error!("Error receiving initialization message {e:?}");
            }
        }

        Ok(Self {
            decoder,
            cmd_rx,
            player_cmd_tx,
            event_tx,
            last_send_time: Duration::default(),
        })
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

    fn process_input(&mut self) -> bool {
        match self.cmd_rx.try_recv() {
            Ok(command) => {
                info!("Got decoder command {:?}", command);

                match command {
                    DecoderCommand::Play => {
                        self.decoder.resume();
                        if let Err(e) = self.cmd_rx.respond(DecoderResponse::Received) {
                            error!("Error sending stopped response: {e:?}");
                        }
                    }
                    DecoderCommand::Stop => {
                        info!("Completed decoder command");
                        if let Err(e) = self.cmd_rx.respond(DecoderResponse::Received) {
                            error!("Error sending stopped response: {e:?}");
                        }
                        return false;
                    }
                    DecoderCommand::Seek(time) => {
                        let seek_response = match self.decoder.seek(time) {
                            Ok(seeked_to) => Ok(seeked_to.actual_ts),
                            Err(e) => Err(e.to_string()),
                        };
                        if let Err(e) = self
                            .cmd_rx
                            .respond(DecoderResponse::SeekResponse(seek_response))
                        {
                            error!("Unable to send seek result: {e:?}");
                        }
                    }
                    DecoderCommand::Pause => {
                        self.decoder.pause();
                        if let Err(e) = self.cmd_rx.respond(DecoderResponse::Received) {
                            error!("Error sending stopped response: {e:?}");
                        }
                    }
                    DecoderCommand::SetVolume(volume) => {
                        self.decoder.set_volume(volume);
                        if let Err(e) = self.cmd_rx.respond(DecoderResponse::Received) {
                            error!("Error sending set volume response: {e:?}");
                        }
                    }
                    DecoderCommand::GetCurrentPosition => {
                        let time = self.decoder.current_position();
                        if let Err(e) = self
                            .cmd_rx
                            .respond(DecoderResponse::CurrentPositionResponse(time))
                        {
                            error!("Unable to send current position response {e:?}");
                        }
                    }
                    DecoderCommand::WaitForInitialization => {
                        unreachable!("Should only send this during initialization");
                    }
                }
                info!("Completed decoder command");
            }
            Err(TryRecvError::Empty) => {
                let position = self.decoder.current_position();
                if position.position - self.last_send_time >= Duration::from_secs(10) {
                    self.event_tx
                        .send(PlayerEvent::Position(position.clone()))
                        .unwrap_or_default();
                    self.last_send_time = position.position;
                }
            }
            Err(TryRecvError::Disconnected) => {
                info!("Decoder command sender has disconnected");
                return false;
            }
        }

        true
    }

    pub(crate) fn current(&self) -> &[f64] {
        self.decoder.current()
    }

    pub(crate) fn next(&mut self) -> Result<Option<&[f64]>, DecoderError> {
        if !self.process_input() {
            return Ok(None);
        }
        match self.decoder.next() {
            val @ Ok(Some(_)) => val,
            val => {
                if let Err(e) = self.player_cmd_tx.send(Command::Ended) {
                    error!("Unable to send ended command: {e:?}");
                }
                val
            }
        }
    }
}
