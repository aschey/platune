use crate::audio_output::*;
use std::result;
use std::sync::Arc;
use std::{fmt::Debug, time::Duration};
use symphonia::core::audio::RawSample;
use symphonia::core::conv::{ConvertibleSample, FromSample};
use tap::TapFallible;
use thiserror::Error;

pub(crate) trait AudioOutput {
    fn write(&mut self, sample_iter: &[f64]);
    fn stop(&mut self);
    fn start(&mut self) -> Result<()>;
    fn sample_rate(&self) -> usize;
    fn channels(&self) -> usize;
    fn set_device_name(&mut self, name: Option<String>);
}

#[derive(Debug, Error)]
pub enum AudioOutputError {
    #[error("No default device found")]
    NoDefaultDevice,
    #[error("Error getting default device name: {0}")]
    InvalidDefaultDeviceName(DeviceNameError),
    #[error("Error getting default device config: {0}")]
    OutputDeviceConfigError(DefaultStreamConfigError),
    #[error("Error opening output stream: {0}")]
    OpenStreamError(BuildStreamError),
    #[error("Error starting stream: {0}")]
    StartStreamError(PlayStreamError),
    #[error("Unsupported device configuration: {0}")]
    UnsupportedConfiguration(String),
    #[error("Error loading devices: {0}")]
    LoadDevicesError(DevicesError),
}

pub type Result<T> = result::Result<T, AudioOutputError>;

#[derive(PartialEq, Eq)]
enum WriteBufResult {
    Stop,
    Continue,
}

use rb::*;

use tracing::{error, info};

use crate::{
    dto::{command::Command, player_response::PlayerResponse},
    two_way_channel::TwoWaySender,
};

pub(crate) struct CpalAudioOutput;

trait AudioOutputSample:
    cpal::SizedSample + ConvertibleSample + RawSample + Send + Debug + 'static
{
}

impl AudioOutputSample for f32 {}
impl AudioOutputSample for i16 {}
impl AudioOutputSample for u16 {}
impl AudioOutputSample for i8 {}
impl AudioOutputSample for i32 {}
impl AudioOutputSample for u8 {}
impl AudioOutputSample for u32 {}
impl AudioOutputSample for f64 {}

impl CpalAudioOutput {
    pub(crate) fn new_output(
        host: Arc<Host>,
        cmd_sender: TwoWaySender<Command, PlayerResponse>,
        device_name: Option<String>,
    ) -> Result<Box<dyn AudioOutput>> {
        // Get the default audio output device.
        let device = host
            .default_output_device()
            .ok_or(AudioOutputError::NoDefaultDevice)?;
        info!("Using device: {:?}", device.name());
        let config = match device.default_output_config() {
            Ok(config) => config,
            Err(e) => {
                return Err(AudioOutputError::OutputDeviceConfigError(e));
            }
        };
        info!("Device config: {config:?}");

        // Select proper playback routine based on sample format.
        Ok(match config.sample_format() {
            cpal::SampleFormat::F32 => Box::new(CpalAudioOutputImpl::<f32>::new(
                cmd_sender,
                host,
                device_name,
            )),
            cpal::SampleFormat::I16 => Box::new(CpalAudioOutputImpl::<i16>::new(
                cmd_sender,
                host,
                device_name,
            )),
            cpal::SampleFormat::U16 => Box::new(CpalAudioOutputImpl::<u16>::new(
                cmd_sender,
                host,
                device_name,
            )),
            cpal::SampleFormat::I8 => Box::new(CpalAudioOutputImpl::<i8>::new(
                cmd_sender,
                host,
                device_name,
            )),
            cpal::SampleFormat::I32 => Box::new(CpalAudioOutputImpl::<i32>::new(
                cmd_sender,
                host,
                device_name,
            )),
            cpal::SampleFormat::U8 => Box::new(CpalAudioOutputImpl::<u8>::new(
                cmd_sender,
                host,
                device_name,
            )),
            cpal::SampleFormat::U32 => Box::new(CpalAudioOutputImpl::<u32>::new(
                cmd_sender,
                host,
                device_name,
            )),
            cpal::SampleFormat::F64 => Box::new(CpalAudioOutputImpl::<f64>::new(
                cmd_sender,
                host,
                device_name,
            )),
            cpal::SampleFormat::I64 => {
                return Err(AudioOutputError::UnsupportedConfiguration(
                    "Unsupported sample format: i64".to_owned(),
                ))?
            }
            cpal::SampleFormat::U64 => {
                return Err(AudioOutputError::UnsupportedConfiguration(
                    "Unsupported sample format: u64".to_owned(),
                ))?
            }
            _ => {
                return Err(AudioOutputError::UnsupportedConfiguration(
                    "Unsupported sample format: unknown".to_owned(),
                ))?
            }
        })
    }
}

struct CpalAudioOutputImpl<T: AudioOutputSample>
where
    T: AudioOutputSample,
{
    ring_buf_producer: Option<rb::Producer<T>>,
    stream: Option<Stream>,
    sample_rate: usize,
    channels: usize,
    buf: Vec<T>,
    cmd_sender: TwoWaySender<Command, PlayerResponse>,
    host: Arc<Host>,
    device_name: Option<String>,
}

impl<T: AudioOutputSample> CpalAudioOutputImpl<T> {
    pub fn new(
        cmd_sender: TwoWaySender<Command, PlayerResponse>,
        host: Arc<Host>,
        device_name: Option<String>,
    ) -> Self {
        Self {
            ring_buf_producer: None,
            stream: None,
            sample_rate: 0,
            channels: 0,
            buf: vec![T::MID; 2048],
            cmd_sender,
            host,
            device_name,
        }
    }

    fn create_stream(
        device: &Device,
        supported_config: SupportedStreamConfig,
        sample_rate: SampleRate,
        ring_buf_consumer: Consumer<T>,
        cmd_sender: TwoWaySender<Command, PlayerResponse>,
    ) -> Result<Stream> {
        // Output audio stream config.
        let channels = supported_config.channels();
        let config = StreamConfig {
            channels: supported_config.channels(),
            sample_rate,
            buffer_size: cpal::BufferSize::Default,
        };
        info!("Output channels = {channels}");
        info!("Output sample rate = {}", sample_rate.0);

        // Use max value for tests so these can be filtered out later
        #[cfg(test)]
        let filler = <T as FromSample<f32>>::from_sample(f32::MAX);
        #[cfg(not(test))]
        let filler = T::MID;

        let stream_result = device.build_output_stream(
            &config,
            move |data: &mut [T], _: &OutputCallbackInfo| {
                // Write out as many samples as possible from the ring buffer to the audio output.
                let written = ring_buf_consumer.read(data).unwrap_or(0);
                // Mute any remaining samples.
                data[written..].iter_mut().for_each(|s| *s = filler);
            },
            move |err| match err {
                StreamError::DeviceNotAvailable => {
                    info!("Device unplugged. Resetting...");
                    let _ = cmd_sender
                        .send(Command::Reset)
                        .tap_err(|e| error!("Error sending reset command: {e:?}"));
                }
                StreamError::BackendSpecific { err } => {
                    error!("Playback error: {err}");
                    let _ = cmd_sender
                        .send(Command::Stop)
                        .tap_err(|e| error!("Error sending stop command: {e:?}"));
                }
            },
            None,
        );

        let stream = stream_result.map_err(AudioOutputError::OpenStreamError)?;

        // Start the output stream.
        stream.play().map_err(AudioOutputError::StartStreamError)?;

        Ok(stream)
    }

    fn write_buf(&mut self, end_index: Option<usize>) -> WriteBufResult {
        if let Some(ring_buf_producer) = &mut self.ring_buf_producer {
            let mut samples = match end_index {
                Some(end_index) => &self.buf[..end_index],
                None => &self.buf[..],
            };
            loop {
                match ring_buf_producer.write_blocking_timeout(samples, Duration::from_millis(1000))
                {
                    Ok(Some(written)) => {
                        samples = &samples[written..];
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(_) => {
                        info!("Consumer stalled. Terminating.");
                        return WriteBufResult::Stop;
                    }
                }
            }
        }
        WriteBufResult::Continue
    }
}

impl<T: AudioOutputSample> AudioOutput for CpalAudioOutputImpl<T> {
    fn write(&mut self, sample_iter: &[f64]) {
        let mut i = 0;

        for frame in sample_iter {
            if i == self.buf.len() {
                if self.write_buf(None) == WriteBufResult::Stop {
                    return;
                }

                i = 0;
            }

            self.buf[i] = <T as FromSample<f64>>::from_sample(*frame);
            i += 1;
        }

        self.write_buf(Some(i));
    }

    fn channels(&self) -> usize {
        self.channels
    }

    fn stop(&mut self) {
        self.stream = None;
    }

    fn set_device_name(&mut self, name: Option<String>) {
        self.device_name = name;
    }

    fn start(&mut self) -> Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }

        let default_device = self
            .host
            .default_output_device()
            .ok_or(AudioOutputError::NoDefaultDevice)?;

        // Get the default audio output device.
        let chosen_device_name = match &self.device_name {
            Some(name) => name.to_owned(),
            None => default_device
                .name()
                .map_err(AudioOutputError::InvalidDefaultDeviceName)?,
        };

        // We explicitly need to select a device here rather than using the default.
        // This is because on Mac if we use the default device, we won't receive a disconnect error and the device will change automatically.
        // This is sometimes fine except when the sample rate on the new device is different, we need to restart the stream to resample correctly.
        // More info here: https://github.com/RustAudio/cpal/pull/707#issuecomment-1275609798
        let device = self
            .host
            .devices()
            .map_err(AudioOutputError::LoadDevicesError)?
            .find(|d| {
                d.name()
                    .map(|n| n.trim() == chosen_device_name.trim())
                    .unwrap_or(false)
            })
            // Fall back to default device if chosen device was not found
            .unwrap_or(default_device);

        let config = device
            .default_output_config()
            .map_err(AudioOutputError::OutputDeviceConfigError)?;

        let ring_buf = SpscRb::<T>::new(8 * 1024);
        let (ring_buf_producer, ring_buf_consumer) = (ring_buf.producer(), ring_buf.consumer());

        let sample_rate = config.sample_rate();
        self.sample_rate = sample_rate.0 as usize;
        let channels = config.channels() as usize;
        if !(1..=2).contains(&channels) {
            return Err(AudioOutputError::UnsupportedConfiguration(format!(
                "Outputs with {channels} channels are not supported"
            )));
        }
        self.channels = channels;

        let stream = match Self::create_stream(
            &device,
            config,
            sample_rate,
            ring_buf_consumer,
            self.cmd_sender.clone(),
        ) {
            Ok(stream) => stream,
            Err(e) => return Err(e),
        };

        self.ring_buf_producer = Some(ring_buf_producer);
        self.stream = Some(stream);

        Ok(())
    }

    fn sample_rate(&self) -> usize {
        self.sample_rate
    }
}
