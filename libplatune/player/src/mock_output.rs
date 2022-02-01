use std::thread;
use std::time::Duration;

use cpal::{traits::StreamTrait, BufferSize, ChannelCount, InputDevices, OutputDevices, Sample};
use cpal::{
    BuildStreamError, DefaultStreamConfigError, DeviceNameError, DevicesError, InputCallbackInfo,
    PauseStreamError, PlayStreamError, SampleFormat, SampleRate, StreamConfig, StreamError,
    SupportedBufferSize, SupportedStreamConfigRange, SupportedStreamConfigsError,
};
use flume::Sender;
use spin_sleep::SpinSleeper;

pub fn default_host() -> Host {
    Host::new().unwrap()
}

#[derive(Debug, Clone, PartialEq)]
pub struct OutputCallbackInfo {
    timestamp: OutputStreamTimestamp,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct StreamInstant {
    secs: i64,
    nanos: u32,
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct OutputStreamTimestamp {
    /// The instant the stream's data callback was invoked.
    pub callback: StreamInstant,
    /// The predicted instant that data written will be delivered to the device for playback.
    ///
    /// E.g. The instant data will be played by a DAC.
    pub playback: StreamInstant,
}

pub struct SupportedStreamConfig {
    channels: ChannelCount,
    sample_rate: SampleRate,
    buffer_size: SupportedBufferSize,
    sample_format: SampleFormat,
}

impl SupportedStreamConfig {
    pub fn channels(&self) -> ChannelCount {
        self.channels
    }

    pub fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    pub fn buffer_size(&self) -> &SupportedBufferSize {
        &self.buffer_size
    }

    pub fn sample_format(&self) -> SampleFormat {
        self.sample_format
    }

    pub fn config(&self) -> StreamConfig {
        StreamConfig {
            channels: self.channels,
            sample_rate: self.sample_rate,
            buffer_size: BufferSize::Default,
        }
    }
}

pub trait HostTrait {
    /// The type used for enumerating available devices by the host.
    type Devices: Iterator<Item = Self::Device>;
    /// The `Device` type yielded by the host.
    type Device: DeviceTrait;

    /// Whether or not the host is available on the system.
    fn is_available() -> bool;

    /// An iterator yielding all `Device`s currently available to the host on the system.
    ///
    /// Can be empty if the system does not support audio in general.
    fn devices(&self) -> Result<Self::Devices, DevicesError>;

    /// The default input audio device on the system.
    ///
    /// Returns `None` if no input device is available.
    fn default_input_device(&self) -> Option<Self::Device>;

    /// The default output audio device on the system.
    ///
    /// Returns `None` if no output device is available.
    fn default_output_device(&self) -> Option<Self::Device>;

    /// An iterator yielding all `Device`s currently available to the system that support one or more
    /// input stream formats.
    ///
    /// Can be empty if the system does not support audio input.
    fn input_devices(&self) -> Result<InputDevices<Self::Devices>, DevicesError> {
        fn supports_input<D: DeviceTrait>(device: &D) -> bool {
            device
                .supported_input_configs()
                .map(|mut iter| iter.next().is_some())
                .unwrap_or(false)
        }
        Ok(self.devices()?.filter(supports_input::<Self::Device>))
    }

    /// An iterator yielding all `Device`s currently available to the system that support one or more
    /// output stream formats.
    ///
    /// Can be empty if the system does not support audio output.
    fn output_devices(&self) -> Result<OutputDevices<Self::Devices>, DevicesError> {
        fn supports_output<D: DeviceTrait>(device: &D) -> bool {
            device
                .supported_output_configs()
                .map(|mut iter| iter.next().is_some())
                .unwrap_or(false)
        }
        Ok(self.devices()?.filter(supports_output::<Self::Device>))
    }
}

pub trait DeviceTrait {
    /// The iterator type yielding supported input stream formats.
    type SupportedInputConfigs: Iterator<Item = SupportedStreamConfigRange>;
    /// The iterator type yielding supported output stream formats.
    type SupportedOutputConfigs: Iterator<Item = SupportedStreamConfigRange>;
    /// The stream type created by `build_input_stream_raw` and `build_output_stream_raw`.
    type Stream: StreamTrait;

    /// The human-readable name of the device.
    fn name(&self) -> Result<String, DeviceNameError>;

    /// An iterator yielding formats that are supported by the backend.
    ///
    /// Can return an error if the device is no longer valid (e.g. it has been disconnected).
    fn supported_input_configs(
        &self,
    ) -> Result<Self::SupportedInputConfigs, SupportedStreamConfigsError>;

    /// An iterator yielding output stream formats that are supported by the device.
    ///
    /// Can return an error if the device is no longer valid (e.g. it has been disconnected).
    fn supported_output_configs(
        &self,
    ) -> Result<Self::SupportedOutputConfigs, SupportedStreamConfigsError>;

    /// The default input stream format for the device.
    fn default_input_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError>;

    /// The default output stream format for the device.
    fn default_output_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError>;

    /// Create an input stream.
    fn build_input_stream<T, D, E>(
        &self,
        config: &StreamConfig,
        data_callback: D,
        error_callback: E,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        T: Sample,
        D: FnMut(&[T], &InputCallbackInfo) + Send + 'static,
        E: FnMut(StreamError) + Send + 'static;

    /// Create an output stream.
    fn build_output_stream<T, D, E>(
        &self,
        config: &StreamConfig,
        data_callback: D,
        error_callback: E,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        T: Sample,
        D: FnMut(&mut [T], &OutputCallbackInfo) + Send + 'static,
        E: FnMut(StreamError) + Send + 'static;
}

#[derive(Default)]
pub struct Devices;

pub struct Host {
    output_device: Device,
}

#[derive(Debug)]
pub struct Stream {
    shutdown_tx: Option<Sender<()>>,
}

pub struct SupportedInputConfigs;
pub struct SupportedOutputConfigs;

impl Host {
    pub fn new_with_options(
        audio_sleep_time: Duration,
        sample_rate: u32,
    ) -> Result<Self, cpal::HostUnavailable> {
        Ok(Host {
            output_device: Device::new(Some(audio_sleep_time), sample_rate),
        })
    }

    pub fn new() -> Result<Self, cpal::HostUnavailable> {
        Ok(Host {
            output_device: Device::new(None, 44_100),
        })
    }
}

impl Devices {
    pub fn new() -> Result<Self, DevicesError> {
        Ok(Devices)
    }
}

#[derive(Clone, Debug)]
pub struct Device {
    data_tx: tokio::sync::broadcast::Sender<Vec<f32>>,
    audio_sleep_time: Option<Duration>,
    sample_rate: u32,
}

impl Device {
    pub fn new(audio_sleep_time: Option<Duration>, sample_rate: u32) -> Self {
        let (data_tx, _) = tokio::sync::broadcast::channel(2048);
        Self {
            data_tx,
            audio_sleep_time,
            sample_rate,
        }
    }

    pub fn subscribe_data(&self) -> tokio::sync::broadcast::Receiver<Vec<f32>> {
        self.data_tx.subscribe()
    }
}

impl DeviceTrait for Device {
    type SupportedInputConfigs = SupportedInputConfigs;
    type SupportedOutputConfigs = SupportedOutputConfigs;
    type Stream = Stream;

    #[inline]
    fn name(&self) -> Result<String, DeviceNameError> {
        Ok("null".to_owned())
    }

    #[inline]
    fn supported_input_configs(
        &self,
    ) -> Result<SupportedInputConfigs, SupportedStreamConfigsError> {
        Ok(SupportedInputConfigs {})
    }

    #[inline]
    fn supported_output_configs(
        &self,
    ) -> Result<SupportedOutputConfigs, SupportedStreamConfigsError> {
        Ok(SupportedOutputConfigs {})
    }

    #[inline]
    fn default_input_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError> {
        Ok(SupportedStreamConfig {
            channels: 2,
            sample_rate: SampleRate(44100),
            buffer_size: SupportedBufferSize::Range {
                min: 0,
                max: u32::MAX,
            },
            sample_format: SampleFormat::F32,
        })
    }

    #[inline]
    fn default_output_config(&self) -> Result<SupportedStreamConfig, DefaultStreamConfigError> {
        Ok(SupportedStreamConfig {
            channels: 2,
            sample_rate: SampleRate(self.sample_rate),
            buffer_size: SupportedBufferSize::Range {
                min: 0,
                max: u32::MAX,
            },
            sample_format: SampleFormat::F32,
        })
    }

    fn build_input_stream<T, D, E>(
        &self,
        _config: &StreamConfig,
        mut _data_callback: D,
        _error_callback: E,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        T: Sample,
        D: FnMut(&[T], &InputCallbackInfo) + Send + 'static,
        E: FnMut(StreamError) + Send + 'static,
    {
        unimplemented!();
    }

    fn build_output_stream<T, D, E>(
        &self,
        _config: &StreamConfig,
        mut data_callback: D,
        _error_callback: E,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        T: Sample,
        D: FnMut(&mut [T], &OutputCallbackInfo) + Send + 'static,
        E: FnMut(StreamError) + Send + 'static,
    {
        let (shutdown_tx, shutdown_rx) = flume::bounded(1);
        let data_tx = self.data_tx.clone();
        let sleep_time = self.audio_sleep_time;
        thread::spawn(move || {
            const BUF_SIZE: usize = 1024;
            let mut buf = [0f32; BUF_SIZE];

            let mut data =
                unsafe { std::slice::from_raw_parts(buf.as_mut_ptr() as *const T, BUF_SIZE) }
                    .to_owned();
            let info = OutputCallbackInfo {
                timestamp: OutputStreamTimestamp {
                    callback: StreamInstant { secs: 0, nanos: 0 },
                    playback: StreamInstant { secs: 0, nanos: 0 },
                },
            };
            // Using thread::sleep on Windows seems to be widely inaccurate at scale
            let spin_sleeper = SpinSleeper::new(100_000);
            let mut shutdown_requested = false;
            loop {
                data_callback(&mut data, &info);
                let data_f32: Vec<f32> = data
                    .iter()
                    .map(|d| d.to_f32())
                    // f32::MAX means the sample wasn't written in time, filter these for testing purposes
                    // Since many tests run in parallel this is likely to happen sometimes
                    .filter(|d| *d != f32::MAX)
                    .collect();
                // Shutdown once no more data is received
                if shutdown_requested && data_f32.iter().all(|d| *d == 0.0) {
                    break;
                }
                data_tx.send(data_f32).unwrap_or_default();
                if let Some(sleep_time) = sleep_time {
                    spin_sleeper.sleep(sleep_time);
                }
                if let Ok(()) = shutdown_rx.try_recv() {
                    shutdown_requested = true;
                }
            }
        });

        Ok(Self::Stream {
            shutdown_tx: Some(shutdown_tx),
        })
    }
}

impl Drop for Stream {
    #[inline]
    fn drop(&mut self) {
        if let Some(sender) = self.shutdown_tx.take() {
            sender.send(()).unwrap_or_default();
        }
    }
}

impl HostTrait for Host {
    type Device = Device;
    type Devices = Devices;

    fn is_available() -> bool {
        true
    }

    fn devices(&self) -> Result<Self::Devices, DevicesError> {
        Devices::new()
    }

    fn default_input_device(&self) -> Option<Device> {
        Some(Device::new(None, 44_100))
    }

    fn default_output_device(&self) -> Option<Device> {
        Some(self.output_device.clone())
    }
}

impl StreamTrait for Stream {
    fn play(&self) -> Result<(), PlayStreamError> {
        Ok(())
    }

    fn pause(&self) -> Result<(), PauseStreamError> {
        Ok(())
    }
}

impl Iterator for Devices {
    type Item = Device;

    #[inline]
    fn next(&mut self) -> Option<Device> {
        None
    }
}

impl Iterator for SupportedInputConfigs {
    type Item = SupportedStreamConfigRange;

    #[inline]
    fn next(&mut self) -> Option<SupportedStreamConfigRange> {
        None
    }
}

impl Iterator for SupportedOutputConfigs {
    type Item = SupportedStreamConfigRange;

    #[inline]
    fn next(&mut self) -> Option<SupportedStreamConfigRange> {
        None
    }
}
