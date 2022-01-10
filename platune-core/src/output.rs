use cpal::{SampleRate, Stream};
use std::ops::Mul;
use std::result;
use symphonia::core::audio::{AudioBufferRef, RawSample, SampleBuffer, SignalSpec};
use symphonia::core::conv::ConvertibleSample;
use symphonia::core::units::Duration;

pub trait AudioOutput {
    fn write(&mut self, decoded: AudioBufferRef<'_>);
    fn write_empty(&mut self);
    fn flush(&mut self);
    fn init_track(&mut self, spec: SignalSpec, duration: Duration);
    fn stop(&mut self);
    fn resume(&mut self);
    fn set_volume(&mut self, volume: f32);
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

pub struct CpalAudioOutput;

trait AudioOutputSample:
    cpal::Sample + ConvertibleSample + RawSample + Mul<Output = Self> + Send + 'static
{
}

impl AudioOutputSample for f32 {}
impl AudioOutputSample for i16 {}
impl AudioOutputSample for u16 {}

impl CpalAudioOutput {
    pub fn try_open() -> Result<Box<dyn AudioOutput>> {
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
        match config.sample_format() {
            cpal::SampleFormat::F32 => CpalAudioOutputImpl::<f32>::try_open(&device, sample_rate),
            cpal::SampleFormat::I16 => CpalAudioOutputImpl::<i16>::try_open(&device, sample_rate),
            cpal::SampleFormat::U16 => CpalAudioOutputImpl::<u16>::try_open(&device, sample_rate),
        }
    }
}

struct CpalAudioOutputImpl<T: AudioOutputSample>
where
    T: AudioOutputSample,
{
    ring_buf_producer: rb::Producer<T>,
    sample_buf: Option<SampleBuffer<T>>,
    stream: Option<cpal::Stream>,
    silence_skipped: bool,
    volume: T,
    buf: Vec<T>,
}

impl<T: AudioOutputSample> CpalAudioOutputImpl<T> {
    pub fn try_open(
        device: &cpal::Device,
        sample_rate: SampleRate,
    ) -> Result<Box<dyn AudioOutput>> {
        // Instantiate a ring buffer capable of buffering 8K (arbitrarily chosen) samples.
        let ring_buf = SpscRb::<T>::new(8 * 1024);
        let (ring_buf_producer, ring_buf_consumer) = (ring_buf.producer(), ring_buf.consumer());

        match Self::create_stream(device, sample_rate, ring_buf_consumer) {
            Ok(stream) => Ok(Box::new(CpalAudioOutputImpl {
                ring_buf_producer,
                sample_buf: None,
                stream: Some(stream),
                silence_skipped: false,
                buf: Vec::<T>::new(),
                volume: T::from(&1.0),
            })),
            Err(e) => Err(e),
        }
    }

    fn create_stream(
        device: &cpal::Device,
        sample_rate: SampleRate,
        ring_buf_consumer: Consumer<T>,
    ) -> Result<Stream> {
        // Output audio stream config.
        let supported_config = device.default_output_config().unwrap();
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
    fn write(&mut self, decoded: AudioBufferRef<'_>) {
        // Do nothing if there are no audio frames.
        if decoded.frames() == 0 {
            return;
        }

        // Audio samples must be interleaved for cpal. Interleave the samples in the audio
        // buffer into the sample buffer.
        if let Some(sample_buf) = &mut self.sample_buf {
            sample_buf.copy_interleaved_ref(decoded);
            // Write all the interleaved samples to the ring buffer.
            let mut samples = sample_buf.samples();

            if !self.silence_skipped {
                match samples.iter().position(|s| *s != T::MID) {
                    Some(index) => {
                        info!("Skipped {} silent samples", index);
                        samples = &samples[index..];
                        self.silence_skipped = true;
                    }
                    None => return,
                }
            }
            if samples.len() > self.buf.len() {
                self.buf.clear();
                self.buf.resize(samples.len(), T::default());
            }

            for (i, sample) in samples.iter().enumerate() {
                self.buf[i] = *sample * self.volume;
            }
            samples = &self.buf[..samples.len()];
            while let Some(written) = self.ring_buf_producer.write_blocking(samples) {
                samples = &samples[written..];
            }
        }
    }

    fn write_empty(&mut self) {
        self.ring_buf_producer.write_blocking(&[T::MID]);
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

    fn resume(&mut self) {
        if self.stream.is_some() {
            return;
        }

        let host = cpal::default_host();
        // Get the default audio output device.
        let device = host.default_output_device().unwrap();
        let ring_buf = SpscRb::<T>::new(8 * 1024);
        let (ring_buf_producer, ring_buf_consumer) = (ring_buf.producer(), ring_buf.consumer());
        let stream = Self::create_stream(
            &device,
            device.default_output_config().unwrap().sample_rate(),
            ring_buf_consumer,
        );
        self.ring_buf_producer = ring_buf_producer;
        self.stream = Some(stream.unwrap());
    }

    fn init_track(&mut self, spec: SignalSpec, duration: Duration) {
        self.sample_buf = Some(SampleBuffer::<T>::new(duration, spec));
        self.silence_skipped = false;
    }

    fn set_volume(&mut self, volume: f32) {
        self.volume = T::from(&volume);
    }
}
