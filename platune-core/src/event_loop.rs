use crate::{
    audio_processor::AudioProcessor,
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse, queue_source::QueueSource,
    },
    output::{AudioOutput, CpalAudioOutput},
    player::Player,
    source::Source,
    TwoWayReceiver, TwoWayReceiverAsync, TwoWaySenderAsync,
};
use crossbeam_channel::{Receiver, TryRecvError};
use rubato::{FftFixedInOut, Resampler};
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
                enable_resampling,
                force_restart_output,
            }) => {
                if force_restart_output {
                    audio_manager.stop();
                    audio_manager.reset();
                } else {
                    audio_manager.start();
                }
                audio_manager.decode_source(source, &mut cmd_rx, &player_cmd_tx, enable_resampling);
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
                        audio_manager.reset();
                        audio_manager.decode_source(
                            source,
                            &mut cmd_rx,
                            &player_cmd_tx,
                            enable_resampling,
                        );
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
    in_buf: Vec<Vec<f64>>,
    out_buf: Vec<f64>,
    input_sample_rate: usize,
    output: Box<dyn AudioOutput>,
    buf_index: usize,
    resampler: FftFixedInOut<f64>,
    volume: f64,
}

impl AudioManager {
    fn new(output: Box<dyn AudioOutput>, volume: f64) -> Self {
        let default_sample_rate = 44_100;
        let default_channels = 2;
        let resampler = FftFixedInOut::<f64>::new(
            default_sample_rate,
            default_sample_rate,
            1024,
            default_channels,
        );

        let n_frames = resampler.nbr_frames_needed();
        Self {
            in_buf: vec![vec![0.0; n_frames]; default_channels],
            out_buf: vec![0.0; n_frames * default_channels],
            input_sample_rate: default_sample_rate,
            buf_index: 0,
            resampler,
            output,
            volume,
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
        let channels = self.output.channels();

        self.in_buf = vec![vec![0.0; n_frames]; channels];
        self.out_buf = vec![0.0; n_frames * channels];
    }

    fn reset(&mut self) {
        self.output.start();
        self.set_resampler();
    }

    fn start(&mut self) {
        self.output.start();
    }

    fn stop(&mut self) {
        self.output.stop();
    }

    fn play_remaining(&mut self) {
        if self.buf_index > 0 {
            for i in self.buf_index..self.in_buf[0].len() {
                for chan in &mut self.in_buf {
                    chan[i] = 0.0;
                }
            }
            self.write_output();
        }
    }

    fn write_output(&mut self) {
        self.buf_index = 0;
        let resampled = self.resampler.process(&self.in_buf).unwrap();
        let out_len = resampled[0].len();
        if out_len * self.output.channels() > self.out_buf.len() {
            self.out_buf.clear();
            self.out_buf.resize(out_len * self.output.channels(), 0.0);
        }

        let mut j = 0;
        for i in 0..out_len {
            for chan in &resampled {
                self.out_buf[j] = chan[i];
                j += 1;
            }
        }

        self.output.write(&self.out_buf);
    }

    fn decode_source(
        &mut self,
        source: Box<dyn Source>,
        cmd_rx: &mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
        player_cmd_tx: &TwoWaySenderAsync<Command, PlayerResponse>,
        enable_resampling: bool,
    ) {
        let mut processor = AudioProcessor::new(
            source,
            self.output.channels(),
            cmd_rx,
            player_cmd_tx,
            self.volume,
        );

        let input_sample_rate = processor.sample_rate();
        if input_sample_rate != self.input_sample_rate {
            self.play_remaining();
            self.input_sample_rate = input_sample_rate;
            self.set_resampler();
        }

        if enable_resampling && processor.sample_rate() != self.output.sample_rate() {
            self.decode_resample(&mut processor);
        } else {
            self.decode_no_resample(&mut processor);
        }
        self.volume = processor.volume();
    }

    fn decode_no_resample(&mut self, processor: &mut AudioProcessor) {
        while let Some(next) = processor.next() {
            self.output.write(next);
        }
    }

    fn decode_resample(&mut self, processor: &mut AudioProcessor) {
        let n_frames = self.resampler.nbr_frames_needed();
        let mut frame_pos = 0;

        let mut cur_frame = processor.current();
        loop {
            while self.buf_index < n_frames {
                for chan in &mut self.in_buf {
                    chan[self.buf_index] = cur_frame[frame_pos];
                    frame_pos += 1;
                }

                self.buf_index += 1;

                if frame_pos == cur_frame.len() {
                    match processor.next() {
                        Some(next) => cur_frame = next,
                        None => return,
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
