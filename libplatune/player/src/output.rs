use crate::audio_output::*;
use std::result;
use std::{fmt::Debug, time::Duration};
use symphonia::core::audio::RawSample;
use symphonia::core::conv::ConvertibleSample;
use thiserror::Error;

pub(crate) trait AudioOutput {
    fn write(&mut self, sample_iter: &[f64]);
    fn stop(&mut self);
    fn start(&mut self) -> Result<()>;
    fn sample_rate(&self) -> usize;
    fn channels(&self) -> usize;
}

#[derive(Debug, Error)]
pub enum AudioOutputError {
    #[error("No default device found")]
    NoDefaultDevice,
    #[error("Error getting default device config: {0}")]
    OutputDeviceConfigError(DefaultStreamConfigError),
    #[error("Error opening output stream: {0}")]
    OpenStreamError(BuildStreamError),
    #[error("Error starting stream: {0}")]
    StartStreamError(PlayStreamError),
    #[error("Unsupported device configuration: {0}")]
    UnsupportedConfiguration(String),
}

pub type Result<T> = result::Result<T, AudioOutputError>;

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
        host: Host,
        cmd_sender: TwoWaySender<Command, PlayerResponse>,
    ) -> Result<Box<dyn AudioOutput>> {
        // Get the default audio output device.
        let device = match host.default_output_device() {
            Some(device) => device,
            None => {
                return Err(AudioOutputError::NoDefaultDevice);
            }
        };

        let config = match device.default_output_config() {
            Ok(config) => config,
            Err(e) => {
                return Err(AudioOutputError::OutputDeviceConfigError(e));
            }
        };

        // Select proper playback routine based on sample format.
        Ok(match config.sample_format() {
            cpal::SampleFormat::F32 => Box::new(CpalAudioOutputImpl::<f32>::new(cmd_sender, host)),
            cpal::SampleFormat::I16 => Box::new(CpalAudioOutputImpl::<i16>::new(cmd_sender, host)),
            cpal::SampleFormat::U16 => Box::new(CpalAudioOutputImpl::<u16>::new(cmd_sender, host)),
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
    host: Host,
}

impl<T: AudioOutputSample> CpalAudioOutputImpl<T> {
    pub fn new(cmd_sender: TwoWaySender<Command, PlayerResponse>, host: Host) -> Self {
        Self {
            ring_buf_producer: None,
            stream: None,
            sample_rate: 0,
            channels: 0,
            buf: vec![T::MID; 2048],
            cmd_sender,
            host,
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

        // Use max value for tests to these can be filtered out later
        #[cfg(test)]
        let filler = T::from_sample(f32::MAX);
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
                    if let Err(e) = cmd_sender.try_send(Command::Reset) {
                        error!("Error sending reset command: {e:?}");
                    }
                }
                StreamError::BackendSpecific { err } => {
                    error!("Playback error: {err}");
                    if let Err(e) = cmd_sender.try_send(Command::Stop) {
                        error!("Error sending stop command: {e:?}");
                    }
                }
            },
        );

        let stream = match stream_result {
            Ok(stream) => stream,
            Err(e) => {
                return Err(AudioOutputError::OpenStreamError(e));
            }
        };

        // Start the output stream.
        if let Err(e) = stream.play() {
            return Err(AudioOutputError::StartStreamError(e));
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
                    Ok(Some(written)) => {
                        samples = &samples[written..];
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(_) => {
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

    fn stop(&mut self) {
        self.stream = None;
    }

    fn start(&mut self) -> Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }

        // Get the default audio output device.
        let device = match self.host.default_output_device() {
            Some(device) => device,
            None => {
                return Err(AudioOutputError::NoDefaultDevice);
            }
        };

        let config = match device.default_output_config() {
            Ok(config) => config,
            Err(e) => {
                return Err(AudioOutputError::OutputDeviceConfigError(e));
            }
        };
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
