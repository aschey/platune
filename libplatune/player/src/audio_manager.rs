use crate::{
    audio_processor::AudioProcessor,
    channel_buffer::ChannelBuffer,
    decoder::DecoderParams,
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse, queue_source::QueueSource,
    },
    output::{AudioOutput, AudioOutputError},
    platune_player::PlayerEvent,
    two_way_channel::{TwoWayReceiver, TwoWaySender},
    vec_ext::VecExt,
};
use rubato::{FftFixedInOut, Resampler};
use std::time::Duration;
use tracing::{error, info};

pub(crate) struct AudioManager {
    in_buf: ChannelBuffer,
    out_buf: Vec<f64>,
    input_sample_rate: usize,
    output: Box<dyn AudioOutput>,
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
            in_buf: ChannelBuffer::new(n_frames, default_channels),
            out_buf: Vec::with_capacity(n_frames * default_channels),
            input_sample_rate: default_sample_rate,
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

        self.in_buf = ChannelBuffer::new(n_frames, channels);
        self.out_buf = Vec::with_capacity(n_frames * channels);
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
        if self.in_buf.position() > 0 {
            self.in_buf.silence_remainder();
            self.write_output();
        }
    }

    fn write_output(&mut self) {
        // This shouldn't panic as long as we calculated the number of channels and frames correctly
        let resampled = self
            .resampler
            .process(self.in_buf.inner())
            .expect("number of frames was not correctly calculated");
        self.in_buf.reset();

        self.out_buf.fill_from_deinterleaved(resampled);
        self.output.write(&self.out_buf);
    }

    pub(crate) fn decode_source(
        &mut self,
        queue_source: QueueSource,
        cmd_rx: &mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
        player_cmd_tx: &TwoWaySender<Command, PlayerResponse>,
        event_tx: &tokio::sync::broadcast::Sender<PlayerEvent>,
        start_position: Option<Duration>,
    ) -> Duration {
        let source_name = format!("{queue_source:?}");
        let settings = queue_source.settings.clone();
        let mut processor = match AudioProcessor::new(
            DecoderParams {
                source: queue_source.source,
                volume: self.volume,
                output_channels: self.output.channels(),
                start_position,
            },
            cmd_rx,
            player_cmd_tx,
            event_tx,
            queue_source.wait_for_response,
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
        let mut cur_frame = processor.current();
        let mut written = 0;

        loop {
            while !self.in_buf.is_full() {
                written += self.in_buf.fill_from_slice(&cur_frame[written..]);

                if written == cur_frame.len() {
                    match processor.next() {
                        Ok(Some(next)) => {
                            cur_frame = next;
                            written = 0;
                        }
                        Ok(None) => return,
                        Err(e) => {
                            error!("Error while decoding: {e:?}");
                            return;
                        }
                    }
                }
            }

            self.write_output();
        }
    }
}
