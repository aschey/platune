use dasp::{
    interpolate::{linear::Linear, sinc::Sinc},
    ring_buffer::Fixed,
    signal::{
        from_interleaved_samples_iter, interpolate::Converter, FromInterleavedSamplesIterator,
        UntilExhausted,
    },
    Frame, Signal,
};
use rubato::{FftFixedIn, FftFixedInOut, Resampler};

use std::{cell::RefCell, rc::Rc};

use crate::{
    audio_processor::{AudioProcessor, AudioProcessorState},
    dto::{command::Command, player_response::PlayerResponse, queue_source::QueueSource},
    output::{AudioOutput, CpalAudioOutput},
    player::Player,
    source::Source,
    stereo_stream::StereoStream,
    TwoWayReceiverAsync,
};
use crossbeam_channel::{Receiver, TryRecvError};
use tracing::{error, info};

pub(crate) fn decode_loop(
    queue_rx: Receiver<QueueSource>,
    processor_state: Rc<RefCell<AudioProcessorState>>,
) {
    let output = CpalAudioOutput::new_output().unwrap();
    let mut audio_manager = AudioManager::new(output);

    loop {
        match queue_rx.try_recv() {
            Ok(QueueSource {
                source,
                enable_resampling,
                force_restart_output,
            }) => {
                if force_restart_output {
                    audio_manager.stop();
                    audio_manager.start();
                }
                let processor = AudioProcessor::new(source, 2, processor_state.clone());
                audio_manager.decode_source(processor, enable_resampling);
            }
            Err(TryRecvError::Empty) => {
                // If no pending source, stop the output to preserve cpu
                audio_manager.play_remaining();
                audio_manager.stop();
                match queue_rx.recv() {
                    Ok(QueueSource {
                        source,
                        enable_resampling,
                        ..
                    }) => {
                        audio_manager.start();
                        let processor = AudioProcessor::new(source, 2, processor_state.clone());
                        audio_manager.decode_source(processor, enable_resampling);
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

struct AudioManager {
    buf: [Vec<f64>; 2],
    buf2: Vec<f64>,
    input_sample_rate: usize,
    output: Box<dyn AudioOutput>,
    buf_index: usize,
    resampler: FftFixedInOut<f64>,
}

impl AudioManager {
    fn new(output: Box<dyn AudioOutput>) -> Self {
        let default_sample_rate = 44_100;
        let resampler =
            FftFixedInOut::<f64>::new(default_sample_rate, default_sample_rate, 1024, 2);

        let n_frames = resampler.nbr_frames_needed();
        Self {
            buf: [vec![0.0; n_frames], vec![0.0; n_frames]],
            buf2: vec![0.0; n_frames * 2],
            input_sample_rate: default_sample_rate,
            buf_index: 0,
            resampler,
            output,
        }
    }

    fn set_resampler(&mut self) {
        self.resampler = FftFixedInOut::<f64>::new(
            self.input_sample_rate,
            self.output.sample_rate(),
            1024,
            self.output.channels(),
        );
        let n_frames = self.resampler.nbr_frames_needed();
        self.buf = [vec![0.0; n_frames], vec![0.0; n_frames]];
        self.buf2 = vec![0.0; n_frames * 2];
    }

    fn start(&mut self) {
        self.output.start();
        self.set_resampler();
    }

    fn stop(&mut self) {
        self.output.stop();
    }

    fn play_remaining(&mut self) {
        if self.buf_index > 0 {
            for i in self.buf_index..self.buf[0].len() {
                self.buf[0][i] = 0.0;
                self.buf[1][i] = 0.0;
            }
            self.write_output();
        }
    }

    fn write_output(&mut self) {
        self.buf_index = 0;
        let resampled = self.resampler.process(&self.buf).unwrap();
        let out_len = resampled[0].len();
        if out_len * 2 > self.buf2.len() {
            self.buf2.clear();
            self.buf2.resize(out_len * 2, 0.0);
        }

        let l = &resampled[0];
        let r = &resampled[1];
        let mut j = 0;
        for i in 0..out_len {
            self.buf2[j] = l[i];
            self.buf2[j + 1] = r[i];
            j += 2;
        }

        self.output.write(&self.buf2);
    }

    fn decode_source(&mut self, processor: AudioProcessor, enable_resampling: bool) {
        let input_sample_rate = processor.sample_rate();
        if input_sample_rate != self.input_sample_rate {
            self.play_remaining();
            self.input_sample_rate = input_sample_rate;
            self.set_resampler();
        }

        if enable_resampling && processor.sample_rate() != self.output.sample_rate() {
            self.decode_resample(processor);
        } else {
            self.decode_no_resample(processor);
        }
    }

    fn decode_no_resample(&mut self, mut processor: AudioProcessor) {
        while let Some(next) = processor.next() {
            self.output.write(next);
        }
    }

    fn decode_resample(&mut self, mut processor: AudioProcessor) {
        let n_frames = self.resampler.nbr_frames_needed();
        processor.next();

        let mut frame_pos = 0;

        loop {
            while self.buf_index < n_frames {
                self.buf[0][self.buf_index] = processor.current()[frame_pos];
                self.buf[1][self.buf_index] = processor.current()[frame_pos + 1];
                self.buf_index += 1;
                frame_pos += 2;
                if frame_pos == processor.current().len() {
                    if processor.next().is_none() {
                        return;
                    }
                    frame_pos = 0;
                }
            }

            self.write_output();
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
