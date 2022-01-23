use cpal::{SampleRate, Stream, StreamError, SupportedStreamConfig};
use std::result;
use std::{fmt::Debug, time::Duration};
use symphonia::core::audio::RawSample;
use symphonia::core::conv::ConvertibleSample;

pub(crate) trait AudioOutput {
    fn write(&mut self, sample_iter: &[f64]);
    fn flush(&mut self);
    fn stop(&mut self);
    fn start(&mut self);
    fn sample_rate(&self) -> usize;
    fn channels(&self) -> usize;
}

#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum AudioOutputError {
    OpenStreamError,
    PlayStreamError,
    StreamClosedError,
}

pub type Result<T> = result::Result<T, AudioOutputError>;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rb::*;

use tracing::{error, info};

use crate::{
    dto::{command::Command, player_response::PlayerResponse},
    two_way_channel::TwoWaySender,
};

pub(crate) struct CpalAudioOutput;

trait AudioOutputSample: cpal::Sample + ConvertibleSample + RawSample + Send + Debug + 'static {}

impl AudioOutputSample for f32 {}
impl AudioOutputSample for i16 {}
impl AudioOutputSample for u16 {}

impl CpalAudioOutput {
    pub(crate) fn new_output(
        cmd_sender: TwoWaySender<Command, PlayerResponse>,
    ) -> Result<Box<dyn AudioOutput>> {
        // Get default host.
        let host = cpal::default_host();

        // Get the default audio output device.
        let device = match host.default_output_device() {
            Some(device) => device,
            _ => {
                error!("failed to get default audio output device");
                return Err(AudioOutputError::OpenStreamError);
            }
        };

        let config = match device.default_output_config() {
            Ok(config) => config,
            Err(err) => {
                error!("failed to get default audio output device config: {}", err);
                return Err(AudioOutputError::OpenStreamError);
            }
        };

        // Select proper playback routine based on sample format.
        Ok(match config.sample_format() {
            cpal::SampleFormat::F32 => Box::new(CpalAudioOutputImpl::<f32>::new(cmd_sender)),
            cpal::SampleFormat::I16 => Box::new(CpalAudioOutputImpl::<i16>::new(cmd_sender)),
            cpal::SampleFormat::U16 => Box::new(CpalAudioOutputImpl::<u16>::new(cmd_sender)),
        })
    }
}

struct CpalAudioOutputImpl<T: AudioOutputSample>
where
    T: AudioOutputSample,
{
    ring_buf_producer: Option<rb::Producer<T>>,
    stream: Option<cpal::Stream>,
    sample_rate: usize,
    channels: usize,
    buf: Vec<T>,
    cmd_sender: TwoWaySender<Command, PlayerResponse>,
}

impl<T: AudioOutputSample> CpalAudioOutputImpl<T> {
    pub fn new(cmd_sender: TwoWaySender<Command, PlayerResponse>) -> Self {
        Self {
            ring_buf_producer: None,
            stream: None,
            sample_rate: 0,
            channels: 0,
            buf: vec![T::MID; 2048],
            cmd_sender,
        }
    }

    fn create_stream(
        device: &cpal::Device,
        supported_config: SupportedStreamConfig,
        sample_rate: SampleRate,
        ring_buf_consumer: Consumer<T>,
        cmd_sender: TwoWaySender<Command, PlayerResponse>,
    ) -> Result<Stream> {
        // Output audio stream config.
        let config = cpal::StreamConfig {
            channels: supported_config.channels(),
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };

        let stream_result = device.build_output_stream(
            &config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                // Write out as many samples as possible from the ring buffer to the audio
                // output.
                let written = ring_buf_consumer.read(data).unwrap_or(0);
                // Mute any remaining samples.
                data[written..].iter_mut().for_each(|s| *s = T::MID);
            },
            move |err| match err {
                StreamError::DeviceNotAvailable => {
                    info!("Device unplugged. Resetting...");
                    cmd_sender.try_send(Command::Reset).unwrap();
                }
                cpal::StreamError::BackendSpecific { err } => {
                    error!("Playback error: {err}");
                    cmd_sender.try_send(Command::Stop).unwrap();
                }
            },
        );

        if let Err(err) = stream_result {
            error!("audio output stream open error: {}", err);

            return Err(AudioOutputError::OpenStreamError);
        }

        let stream = stream_result.unwrap();

        // Start the output stream.
        if let Err(err) = stream.play() {
            error!("audio output stream play error: {}", err);

            return Err(AudioOutputError::PlayStreamError);
        }

        Ok(stream)
    }

    fn write_buf(&mut self, end_index: Option<usize>) -> bool {
        if let Some(ring_buf_producer) = &mut self.ring_buf_producer {
            let mut samples = match end_index {
                Some(end_index) => &self.buf[..end_index],
                None => &self.buf[..],
            };
            loop {
                match ring_buf_producer.write_blocking_timeout(samples, Duration::from_millis(1000))
                {
                    (Some(written), false) => {
                        samples = &samples[written..];
                    }
                    (None, false) => {
                        break;
                    }
                    _ => {
                        info!("Consumer stalled. Terminating.");
                        return true;
                    }
                }
            }
        }
        false
    }
}

impl<T: AudioOutputSample> AudioOutput for CpalAudioOutputImpl<T> {
    fn write(&mut self, sample_iter: &[f64]) {
        let mut i = 0;

        for frame in sample_iter {
            if i == self.buf.len() {
                if self.write_buf(None) {
                    return;
                }

                i = 0;
            }

            self.buf[i] = T::from_sample(*frame);
            i += 1;
        }

        self.write_buf(Some(i));
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn flush(&mut self) {
        // Flush is best-effort, ignore the returned result.
        if let Some(stream) = &self.stream {
            let _ = stream.pause();
            stream.play().unwrap();
        }
    }

    fn stop(&mut self) {
        self.stream = None;
    }

    fn start(&mut self) {
        if self.stream.is_some() {
            return;
        }

        let host = cpal::default_host();
        // Get the default audio output device.
        let device = host.default_output_device().unwrap();
        let ring_buf = SpscRb::<T>::new(8 * 1024);
        let (ring_buf_producer, ring_buf_consumer) = (ring_buf.producer(), ring_buf.consumer());

        let config = device.default_output_config().unwrap();
        let sample_rate = config.sample_rate();
        self.sample_rate = sample_rate.0 as usize;
        self.channels = config.channels() as usize;
        let stream = Self::create_stream(
            &device,
            config,
            sample_rate,
            ring_buf_consumer,
            self.cmd_sender.clone(),
        );

        self.ring_buf_producer = Some(ring_buf_producer);
        self.stream = Some(stream.unwrap());
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }
}
