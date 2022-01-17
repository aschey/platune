use cpal::{SampleRate, Stream, SupportedStreamConfig};
use dasp::{sample::FromSample, Sample};
use std::fmt::Debug;
use std::result;
use symphonia::core::audio::RawSample;
use symphonia::core::conv::ConvertibleSample;

pub trait AudioOutput {
    fn write_stream(&mut self, sample_iter: Box<dyn Iterator<Item = f64>>);
    fn flush(&mut self);
    fn stop(&mut self);
    fn play(&mut self);
    fn sample_rate(&self) -> u32;
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

use tracing::error;

pub struct CpalAudioOutput;

trait AudioOutputSample:
    cpal::Sample + ConvertibleSample + RawSample + FromSample<f64> + Send + Debug + 'static
{
}

impl AudioOutputSample for f32 {}
impl AudioOutputSample for i16 {}
impl AudioOutputSample for u16 {}

impl CpalAudioOutput {
    pub fn new_output() -> Result<Box<dyn AudioOutput>> {
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
        let sample_rate = config.sample_rate();
        // Select proper playback routine based on sample format.
        Ok(match config.sample_format() {
            cpal::SampleFormat::F32 => CpalAudioOutputImpl::<f32>::new_output(device, sample_rate),
            cpal::SampleFormat::I16 => CpalAudioOutputImpl::<i16>::new_output(device, sample_rate),
            cpal::SampleFormat::U16 => CpalAudioOutputImpl::<u16>::new_output(device, sample_rate),
        })
    }
}

struct CpalAudioOutputImpl<T: AudioOutputSample>
where
    T: AudioOutputSample,
{
    ring_buf_producer: rb::Producer<T>,
    stream: Option<cpal::Stream>,
    sample_rate: u32,
    channels: usize,
}

impl<T: AudioOutputSample> CpalAudioOutputImpl<T> {
    pub fn new_output(device: cpal::Device, sample_rate: SampleRate) -> Box<dyn AudioOutput> {
        // Instantiate a ring buffer capable of buffering 8K (arbitrarily chosen) samples.
        let ring_buf = SpscRb::<T>::new(8 * 1024);
        let ring_buf_producer = ring_buf.producer();
        let sample_rate_val = sample_rate.0;

        let config = device.default_output_config().unwrap();

        let channels = config.channels() as usize;

        Box::new(Self {
            ring_buf_producer,
            stream: None,
            sample_rate: sample_rate_val,
            channels,
        })
    }

    fn create_stream(
        device: &cpal::Device,
        supported_config: SupportedStreamConfig,
        sample_rate: SampleRate,
        ring_buf_consumer: Consumer<T>,
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
            move |err| error!("audio output error: {}", err),
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
}

impl<T: AudioOutputSample> AudioOutput for CpalAudioOutputImpl<T> {
    fn write_stream(&mut self, sample_iter: Box<dyn Iterator<Item = f64>>) {
        let mut buf = vec![T::MID; 2048];

        let mut i = 0;

        for frame in sample_iter {
            if i == buf.len() {
                let mut samples = &buf[..];
                while let Some(written) = self.ring_buf_producer.write_blocking(samples) {
                    samples = &samples[written..];
                }
                i = 0;
            }

            buf[i] = frame.to_sample();
            i += 1;
        }

        let mut samples = &buf[..i];

        while let Some(written) = self.ring_buf_producer.write_blocking(samples) {
            samples = &samples[written..];
        }
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

    fn play(&mut self) {
        if self.stream.is_some() {
            return;
        }

        let host = cpal::default_host();
        // Get the default audio output device.
        let device = host.default_output_device().unwrap();
        let ring_buf = SpscRb::<T>::new(8 * 1024);
        let (ring_buf_producer, ring_buf_consumer) = (ring_buf.producer(), ring_buf.consumer());
        let config = device.default_output_config().unwrap();
        let stream = Self::create_stream(
            &device,
            config,
            device.default_output_config().unwrap().sample_rate(),
            ring_buf_consumer,
        );
        self.ring_buf_producer = ring_buf_producer;
        self.stream = Some(stream.unwrap());
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
