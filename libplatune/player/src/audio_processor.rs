use std::time::Duration;

use crate::{
    decoder::Decoder,
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse,
    },
    source::Source,
    two_way_channel::{TwoWayReceiver, TwoWaySender},
};

use flume::TryRecvError;
use tracing::{error, info};

pub(crate) struct AudioProcessor<'a> {
    cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: &'a TwoWaySender<Command, PlayerResponse>,
    decoder: Decoder,
}

impl<'a> AudioProcessor<'a> {
    pub(crate) fn new(
        source: Box<dyn Source>,
        output_channels: usize,
        cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
        player_cmd_tx: &'a TwoWaySender<Command, PlayerResponse>,
        volume: f64,
        start_position: Option<Duration>,
    ) -> Self {
        let decoder = Decoder::new(source, volume, output_channels, start_position);
        Self {
            decoder,
            cmd_rx,
            player_cmd_tx,
        }
    }

    pub(crate) fn sample_rate(&self) -> usize {
        self.decoder.sample_rate()
    }

    pub(crate) fn volume(&self) -> f64 {
        self.decoder.volume()
    }

    pub(crate) fn current_position(&self) -> Duration {
        // This should only return None if the player is stopped which shouldn't happen here
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
                    DecoderCommand::Seek(time) => match self.decoder.seek(time) {
                        Ok(seeked_to) => {
                            if self
                                .cmd_rx
                                .respond(DecoderResponse::SeekResponse(Ok(seeked_to.actual_ts)))
                                .is_err()
                            {
                                error!("Unable to send seek result");
                            }
                        }
                        Err(e) => {
                            if self
                                .cmd_rx
                                .respond(DecoderResponse::SeekResponse(Err(e.to_string())))
                                .is_err()
                            {
                                error!("Unable to send seek result");
                            }
                        }
                    },
                    DecoderCommand::Pause => {
                        self.decoder.pause();
                    }
                    DecoderCommand::SetVolume(volume) => {
                        self.decoder.set_volume(volume);
                    }
                    DecoderCommand::GetCurrentPosition => {
                        let time = self.decoder.current_position();
                        self.cmd_rx
                            .respond(DecoderResponse::CurrentPositionResponse(time))
                            .unwrap();
                    }
                }
                info!("Completed decoder command");
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {}
        }

        true
    }

    pub(crate) fn current(&self) -> &[f64] {
        self.decoder.current()
    }

    pub(crate) fn next(&mut self) -> Option<&[f64]> {
        if !self.process_input() {
            return None;
        }
        match self.decoder.next() {
            Some(val) => Some(val),
            None => {
                self.player_cmd_tx.try_send(Command::Ended).unwrap();
                None
            }
        }
    }
}