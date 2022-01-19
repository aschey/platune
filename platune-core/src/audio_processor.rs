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

pub(crate) struct AudioProcessorState {
    pub(crate) cmd_rx: TwoWayReceiver<DecoderCommand, DecoderResponse>,
    pub(crate) player_cmd_tx: TwoWaySenderAsync<Command, PlayerResponse>,
    pub(crate) volume: f64,
}

impl AudioProcessorState {
    pub(crate) fn new(
        volume: f64,
        cmd_rx: TwoWayReceiver<DecoderCommand, DecoderResponse>,
        player_cmd_tx: TwoWaySenderAsync<Command, PlayerResponse>,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            cmd_rx,
            player_cmd_tx,
            volume,
        }))
    }
}

pub(crate) struct AudioProcessor {
    state: Rc<RefCell<AudioProcessorState>>,
    decoder: Decoder,
    volume: f64,
    iteration: u16,
}

impl AudioProcessor {
    pub(crate) fn new(
        source: Box<dyn Source>,
        output_channels: usize,
        state: Rc<RefCell<AudioProcessorState>>,
    ) -> Self {
        let decoder = Decoder::new(source, output_channels);
        let volume = state.borrow_mut().volume;
        Self {
            decoder,
            state,
            volume,
            iteration: 0,
        }
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.decoder.sample_rate()
    }

    fn process_input(&mut self) -> bool {
        let mut state = self.state.borrow_mut();
        match state.cmd_rx.try_recv() {
            Ok(command) => {
                info!("Got decoder command {:?}", command);

                match command {
                    DecoderCommand::Play => {
                        self.decoder.resume();
                    }
                    DecoderCommand::Stop => {
                        return false;
                    }
                    DecoderCommand::Seek(time) => match self.decoder.seek(time) {
                        Ok(seeked_to) => {
                            if state
                                .cmd_rx
                                .respond(DecoderResponse::SeekResponse(Some(seeked_to.actual_ts)))
                                .is_err()
                            {
                                error!("Unable to send seek result");
                            }
                        }
                        Err(e) => {
                            if state
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
                        state.volume = volume;
                        self.volume = volume;
                    }
                    DecoderCommand::GetCurrentTime => {
                        let time = self.decoder.current_position();
                        state
                            .cmd_rx
                            .respond(DecoderResponse::CurrentTimeResponse(time))
                            .unwrap();
                    }
                }
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
                let state = self.state.borrow_mut();
                state.player_cmd_tx.try_send(Command::Ended).unwrap();
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
