use std::time::Duration;

use rubato::{FftFixedInOut, Resampler};
use tracing::{error, info};

use crate::{
    audio_processor::AudioProcessor,
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse,
    },
    output::{AudioOutput, AudioOutputError},
    platune_player::PlayerEvent,
    settings::Settings,
    source::Source,
    two_way_channel::{TwoWayReceiver, TwoWaySender},
};

pub(crate) struct AudioManager {
    in_buf: Vec<Vec<f64>>,
    out_buf: Vec<f64>,
    input_sample_rate: usize,
    output: Box<dyn AudioOutput>,
    buf_index: usize,
    resampler: FftFixedInOut<f64>,
    volume: f64,
}

impl AudioManager {
    pub(crate) fn new(output: Box<dyn AudioOutput>, volume: f64) -> Self {
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

    fn set_resampler(&mut self, resample_chunk_size: usize) {
        self.resampler = FftFixedInOut::<f64>::new(
            self.input_sample_rate,
            self.output.sample_rate(),
            resample_chunk_size,
            self.output.channels(),
        );
        let n_frames = self.resampler.nbr_frames_needed();
        let channels = self.output.channels();

        self.in_buf = vec![vec![0.0; n_frames]; channels];
        self.out_buf = vec![0.0; n_frames * channels];
    }

    pub(crate) fn reset(&mut self, resample_chunk_size: usize) -> Result<(), AudioOutputError> {
        self.output.start()?;
        self.set_resampler(resample_chunk_size);
        Ok(())
    }

    pub(crate) fn start(&mut self) -> Result<(), AudioOutputError> {
        self.output.start()
    }

    pub(crate) fn stop(&mut self) {
        self.output.stop();
    }

    pub(crate) fn play_remaining(&mut self) {
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
        // This shouldn't panic as long as we calculated the number of channels and frames correctly
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

    pub(crate) fn decode_source(
        &mut self,
        source: Box<dyn Source>,
        cmd_rx: &mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
        player_cmd_tx: &TwoWaySender<Command, PlayerResponse>,
        event_tx: &tokio::sync::broadcast::Sender<PlayerEvent>,
        settings: Settings,
        start_position: Option<Duration>,
    ) -> Duration {
        let source_name = format!("{source:?}");
        let mut processor = match AudioProcessor::new(
            source,
            self.output.channels(),
            cmd_rx,
            player_cmd_tx,
            self.volume,
            start_position,
            event_tx,
        ) {
            Ok(processor) => processor,
            Err(e) => {
                error!("Error creating decoder: {e:?}");
                return Duration::default();
            }
        };

        let input_sample_rate = processor.sample_rate();
        if input_sample_rate != self.input_sample_rate {
            self.play_remaining();
            self.input_sample_rate = input_sample_rate;
            self.set_resampler(settings.resample_chunk_size);
        }

        if settings.enable_resampling && processor.sample_rate() != self.output.sample_rate() {
            info!("Resampling source");
            self.decode_resample(&mut processor);
        } else {
            info!("Not resampling source");
            self.decode_no_resample(&mut processor);
        }
        self.volume = processor.volume();
        info!("Finished decoding {source_name}");
        let stop_position = processor.current_position();
        info!("Stopped decoding at {stop_position:?}");

        stop_position
    }

    fn decode_no_resample(&mut self, processor: &mut AudioProcessor) {
        self.output.write(processor.current());
        loop {
            match processor.next() {
                Ok(Some(data)) => {
                    self.output.write(data);
                }
                Ok(None) => return,
                Err(e) => {
                    error!("Error while decoding: {e:?}");
                    return;
                }
            }
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
                        Ok(Some(next)) => cur_frame = next,
                        Ok(None) => return,
                        Err(e) => {
                            error!("Error while decoding: {e:?}");
                            return;
                        }
                    }

                    frame_pos = 0;
                }
            }

            self.write_output();
        }
    }
}
