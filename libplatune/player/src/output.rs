use crate::audio_output::*;
use rb::{RbConsumer, RbProducer, SpscRb, RB};
use std::result;
use std::sync::{Arc, RwLock};
use std::{fmt::Debug, time::Duration};
use tap::TapFallible;
use thiserror::Error;

pub(crate) struct OutputConfig {
    pub(crate) sample_rate: Option<SampleRate>,
    pub(crate) channels: Option<ChannelCount>,
    pub(crate) sample_format: Option<SampleFormat>,
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
    #[error("Error loading devices: {0}")]
    LoadDevicesError(DevicesError),
    #[error("Error loading config: {0}")]
    LoadConfigsError(SupportedStreamConfigsError),
}

pub type Result<T> = result::Result<T, AudioOutputError>;

use tracing::{error, info, warn};

use crate::{
    dto::{command::Command, player_response::PlayerResponse},
    two_way_channel::TwoWaySender,
};

pub(crate) struct OutputBuilder {
    host: Arc<Host>,
    cmd_sender: TwoWaySender<Command, PlayerResponse>,
    current_device: Arc<RwLock<Option<String>>>,
}

impl OutputBuilder {
    pub(crate) fn new(host: Arc<Host>, cmd_sender: TwoWaySender<Command, PlayerResponse>) -> Self {
        let current_device: Arc<RwLock<Option<_>>> = Default::default();

        #[cfg(windows)]
        {
            let current_device_ = current_device.clone();
            let cmd_sender_ = cmd_sender.clone();
            let host_ = host.clone();
            std::thread::spawn(move || {
                let mut current_default_device = host_
                    .default_output_device()
                    .map(|d| d.name().unwrap_or_default())
                    .unwrap_or_default();
                loop {
                    if current_device_.read().expect("lock poisioned").is_none() {
                        let default_device = host_
                            .default_output_device()
                            .map(|d| d.name().unwrap_or_default())
                            .unwrap_or_default();

                        if default_device != current_default_device {
                            cmd_sender_
                                .send(Command::Reset)
                                .tap_err(|e| error!("Error sending reset command: {e:?}"))
                                .ok();
                            current_default_device = default_device;
                        }
                    }

                    std::thread::sleep(Duration::from_secs(1));
                }
            });
        }

        Self {
            host,
            cmd_sender,
            current_device,
        }
    }

    pub(crate) fn default_output_config(&self) -> Result<SupportedStreamConfig> {
        let device = self
            .host
            .default_output_device()
            .ok_or(AudioOutputError::NoDefaultDevice)?;
        device
            .default_output_config()
            .map_err(AudioOutputError::OutputDeviceConfigError)
    }

    pub(crate) fn find_closest_config(
        &self,
        device_name: Option<String>,
        config: OutputConfig,
    ) -> Result<SupportedStreamConfig> {
        let default_device = self
            .host
            .default_output_device()
            .ok_or(AudioOutputError::NoDefaultDevice)?;
        let device = match &device_name {
            Some(device_name) => self
                .host
                .devices()
                .map_err(AudioOutputError::LoadDevicesError)?
                .find(|d| {
                    d.name()
                        .map(|n| n.trim() == device_name.trim())
                        .unwrap_or(false)
                })
                .unwrap_or(default_device),
            None => default_device,
        };
        let default_config = device
            .default_output_config()
            .map_err(AudioOutputError::OutputDeviceConfigError)?;

        let channels = config.channels.unwrap_or(default_config.channels());
        let sample_rate = config.sample_rate.unwrap_or(default_config.sample_rate());
        let sample_format = config
            .sample_format
            .unwrap_or(default_config.sample_format());

        if default_config.channels() == channels
            && default_config.sample_rate() == sample_rate
            && default_config.sample_format() == sample_format
        {
            return Ok(default_config);
        }

        if let Some(matched_config) = device
            .supported_output_configs()
            .map_err(AudioOutputError::LoadConfigsError)?
            .find(|c| {
                c.channels() == channels
                    && c.sample_format() == sample_format
                    && c.min_sample_rate() <= sample_rate
                    && c.max_sample_rate() >= sample_rate
            })
        {
            return Ok(matched_config.with_sample_rate(sample_rate));
        }

        Ok(default_config)
    }

    pub(crate) fn new_output(
        &self,
        device_name: Option<String>,
        config: SupportedStreamConfig,
    ) -> Result<AudioOutput> {
        *self.current_device.write().expect("lock poisoned") = device_name.clone();
        let default_device = self
            .host
            .default_output_device()
            .ok_or(AudioOutputError::NoDefaultDevice)?;
        let device = match &device_name {
            Some(device_name) => self
                .host
                .devices()
                .map_err(AudioOutputError::LoadDevicesError)?
                .find(|d| {
                    d.name()
                        .map(|n| n.trim() == device_name.trim())
                        .unwrap_or(false)
                })
                .unwrap_or(default_device),
            None => default_device,
        };
        info!("Using device: {:?}", device.name());
        info!("Device config: {config:?}");

        Ok(AudioOutput::new(self.cmd_sender.clone(), device, config))
    }
}

pub(crate) struct AudioOutput {
    ring_buf_producer: Option<rb::Producer<f32>>,
    stream: Option<Stream>,
    cmd_sender: TwoWaySender<Command, PlayerResponse>,
    device: Device,
    config: SupportedStreamConfig,
}

impl AudioOutput {
    pub fn new(
        cmd_sender: TwoWaySender<Command, PlayerResponse>,
        device: Device,
        config: SupportedStreamConfig,
    ) -> Self {
        Self {
            ring_buf_producer: None,
            stream: None,
            cmd_sender,
            device,
            config,
        }
    }

    fn create_stream(&self, ring_buf_consumer: rb::Consumer<f32>) -> Result<Stream> {
        let channels = self.config.channels();
        let config = StreamConfig {
            channels: self.config.channels(),
            sample_rate: self.config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };
        info!("Output channels = {channels}");
        info!("Output sample rate = {}", self.config.sample_rate().0);

        // Use max value for tests so these can be filtered out later
        #[cfg(test)]
        let filler = f32::MAX;

        #[cfg(not(test))]
        let filler = 0.0;
        let cmd_sender = self.cmd_sender.clone();
        let stream_result = self.device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &OutputCallbackInfo| {
                // Write out as many samples as possible from the ring buffer to the audio output.
                let written = ring_buf_consumer.read(data).unwrap_or(0);
                // Mute any remaining samples.
                if data.len() > written {
                    warn!("Output buffer not full, muting remaining");
                    data[written..].iter_mut().for_each(|s| *s = filler);
                }
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

    pub(crate) fn write(&mut self, mut samples: &[f32]) {
        if let Some(producer) = &self.ring_buf_producer {
            loop {
                match producer.write_blocking_timeout(samples, Duration::from_millis(1000)) {
                    Ok(Some(written)) => {
                        samples = &samples[written..];
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(_) => {
                        info!("Consumer stalled. Terminating.");
                        return;
                    }
                }
            }
        }
    }

    pub(crate) fn stop(&mut self) {
        self.stream = None;
    }

    pub(crate) fn start(&mut self) -> Result<()> {
        if self.stream.is_some() {
            return Ok(());
        }

        let buffer_ms = 200;
        let ring_buf = SpscRb::<f32>::new(
            ((buffer_ms * self.config.sample_rate().0 as usize) / 1000)
                * self.config.channels() as usize,
        );
        let (ring_buf_producer, ring_buf_consumer) = (ring_buf.producer(), ring_buf.consumer());

        let channels = self.config.channels() as usize;
        if !(1..=2).contains(&channels) {
            return Err(AudioOutputError::UnsupportedConfiguration(format!(
                "Outputs with {channels} channels are not supported"
            )));
        }

        let stream = match self.create_stream(ring_buf_consumer) {
            Ok(stream) => stream,
            Err(e) => return Err(e),
        };

        self.ring_buf_producer = Some(ring_buf_producer);
        self.stream = Some(stream);

        Ok(())
    }
}
