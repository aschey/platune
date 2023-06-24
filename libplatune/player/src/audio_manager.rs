use crate::output::OutputBuilder;
use crate::{
    audio_output::SupportedStreamConfig,
    audio_processor::AudioProcessor,
    channel_buffer::ChannelBuffer,
    decoder::DecoderParams,
    dto::{
        command::Command, decoder_command::DecoderCommand, decoder_response::DecoderResponse,
        player_response::PlayerResponse, processor_error::ProcessorError,
    },
    output::{AudioOutput, AudioOutputError, OutputConfig},
    platune_player::{PlayerEvent, Settings},
    source::Source,
    two_way_channel::{TwoWayReceiver, TwoWaySender},
    vec_ext::VecExt,
};
use rubato::{FftFixedInOut, Resampler};
use std::time::Duration;
use tracing::{error, info};

pub(crate) struct AudioManager {
    in_buf: ChannelBuffer,
    out_buf: Vec<f32>,
    input_sample_rate: usize,
    output_builder: OutputBuilder,
    device_name: Option<String>,
    output: AudioOutput,
    resampler: FftFixedInOut<f32>,
    resampler_buf: Vec<Vec<f32>>,
    volume: f32,
    output_config: SupportedStreamConfig,
}

impl AudioManager {
    pub(crate) fn new(cpal_output: OutputBuilder, volume: f32) -> Result<Self, AudioOutputError> {
        let output_config = cpal_output.default_output_config()?;
        let sample_rate = output_config.sample_rate().0 as usize;
        let channels = output_config.channels() as usize;

        let resampler = FftFixedInOut::<f32>::new(
            sample_rate,
            sample_rate,
            1024,
            output_config.channels() as usize,
        )
        .expect("failed to create resampler");

        let resampler_buf = resampler.input_buffer_allocate();
        let n_frames = resampler.input_frames_next();
        let output = cpal_output.new_output(None, output_config.clone())?;

        Ok(Self {
            output_builder: cpal_output,
            in_buf: ChannelBuffer::new(n_frames, channels),
            out_buf: Vec::with_capacity(n_frames * channels),
            input_sample_rate: sample_rate,
            resampler,
            resampler_buf,
            output,
            volume,
            output_config,
            device_name: None,
        })
    }

    fn set_resampler(&mut self, resample_chunk_size: usize) {
        self.resampler = FftFixedInOut::<f32>::new(
            self.input_sample_rate,
            self.output_config.sample_rate().0 as usize, // self.output.sample_rate(),
            resample_chunk_size,
            self.output_config.channels() as usize,
        )
        .expect("failed to create resampler");
        let n_frames = self.resampler.input_frames_next();
        self.resampler_buf = self.resampler.input_buffer_allocate();

        let channels = self.output_config.channels() as usize;
        self.in_buf = ChannelBuffer::new(n_frames, channels);
        self.out_buf = Vec::with_capacity(n_frames * channels);
    }

    pub(crate) fn set_device_name(&mut self, device_name: Option<String>) {
        self.device_name = device_name;
    }

    pub(crate) fn reset(
        &mut self,
        output_config: OutputConfig,
        resample_chunk_size: usize,
    ) -> Result<(), AudioOutputError> {
        self.output_config = self
            .output_builder
            .find_closest_config(None, output_config)?;
        self.output = self
            .output_builder
            .new_output(None, self.output_config.clone())?;
        self.set_resampler(resample_chunk_size);
        self.start()?;

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

        self.resampler
            .process_into_buffer(self.in_buf.inner(), &mut self.resampler_buf, None)
            .expect("number of frames was not correctly calculated");
        self.in_buf.reset();

        self.out_buf.fill_from_deinterleaved(&self.resampler_buf);
        self.output.write(&self.out_buf);
    }

    pub(crate) fn initialize_processor<'a>(
        &mut self,
        source: Box<dyn Source>,
        volume: Option<f32>,
        cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
        player_cmd_tx: &'a TwoWaySender<Command, PlayerResponse>,
        event_tx: &'a tokio::sync::broadcast::Sender<PlayerEvent>,
        start_position: Option<Duration>,
    ) -> Result<AudioProcessor<'a>, ProcessorError> {
        AudioProcessor::new(
            DecoderParams {
                source,
                volume: volume.unwrap_or(self.volume),
                output_channels: self.output_config.channels() as usize,
                start_position,
            },
            cmd_rx,
            player_cmd_tx,
            event_tx,
        )
    }

    pub(crate) fn decode_source(
        &mut self,
        processor: &mut AudioProcessor<'_>,
        settings: &Settings,
    ) -> Duration {
        self.volume = processor.volume();

        let input_sample_rate = processor.sample_rate();
        if input_sample_rate != self.input_sample_rate {
            self.play_remaining();
            self.input_sample_rate = input_sample_rate;
            self.set_resampler(settings.resample_chunk_size);
        }

        if processor.sample_rate() != self.output_config.sample_rate().0 as usize {
            info!("Resampling source");
            self.decode_resample(processor);
        } else {
            info!("Not resampling source");
            self.decode_no_resample(processor);
        }
        self.volume = processor.volume();
        info!("Finished decoding");
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
