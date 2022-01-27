use std::time::Duration;

use crate::{
    decoder::{Decoder, DecoderError},
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse,
    },
    platune_player::PlayerEvent,
    source::Source,
    two_way_channel::{TwoWayReceiver, TwoWaySender},
};

use flume::TryRecvError;
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
        source: Box<dyn Source>,
        output_channels: usize,
        cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
        player_cmd_tx: &'a TwoWaySender<Command, PlayerResponse>,
        volume: f64,
        start_position: Option<Duration>,
        event_tx: &'a tokio::sync::broadcast::Sender<PlayerEvent>,
    ) -> Result<Self, DecoderError> {
        let decoder = Decoder::new(source, volume, output_channels, start_position)?;
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
                    }
                    DecoderCommand::Stop => {
                        info!("Completed decoder command");
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
                    }
                    DecoderCommand::SetVolume(volume) => {
                        self.decoder.set_volume(volume);
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
                error!("Decoder command sender has disconnected");
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
                if let Err(e) = self.player_cmd_tx.try_send(Command::Ended) {
                    error!("Unable to send ended command: {e:?}");
                }
                val
            }
        }
    }
}
