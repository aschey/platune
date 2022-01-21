use crate::{
    decoder::Decoder,
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse,
    },
    source::Source,
    TwoWayReceiver, TwoWaySenderAsync,
};
use crossbeam_channel::TryRecvError;
use std::{cell::RefCell, rc::Rc};
use tracing::{error, info};

pub(crate) struct AudioProcessor<'a> {
    cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: &'a TwoWaySenderAsync<Command, PlayerResponse>,
    decoder: Decoder,
}

impl<'a> AudioProcessor<'a> {
    pub(crate) fn new(
        source: Box<dyn Source>,
        output_channels: usize,
        cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
        player_cmd_tx: &'a TwoWaySenderAsync<Command, PlayerResponse>,
        volume: f64,
    ) -> Self {
        let decoder = Decoder::new(source, volume, output_channels);
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
                                .respond(DecoderResponse::SeekResponse(Some(seeked_to.actual_ts)))
                                .is_err()
                            {
                                error!("Unable to send seek result");
                            }
                        }
                        Err(e) => {
                            if self
                                .cmd_rx
                                .respond(DecoderResponse::SeekResponse(None))
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
                    DecoderCommand::GetCurrentTime => {
                        let time = self.decoder.current_position();
                        self.cmd_rx
                            .respond(DecoderResponse::CurrentTimeResponse(time))
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

// impl Iterator for AudioProcessor {
//     type Item = f64;

//     fn next(&mut self) -> Option<Self::Item> {
//         // Reduce checks for user input to save CPU
//         if self.iteration == 2048 {
//             if !self.process_input() {
//                 return None;
//             }
//             self.iteration = 0;
//         } else {
//             self.iteration += 1;
//         }

//         match self.decoder.next() {
//             Some(val) => Some(val * self.volume),
//             None => {
//                 self.process_input();
//                 let state = self.state.borrow_mut();
//                 state.player_cmd_tx.try_send(Command::Ended).unwrap();
//                 None
//             }
//         }
//     }
// }
