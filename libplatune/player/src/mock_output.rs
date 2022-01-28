use std::{
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
};

use cpal::{traits::StreamTrait, BufferSize, ChannelCount, InputDevices, OutputDevices, Sample};
use cpal::{
    BuildStreamError, DefaultStreamConfigError, DeviceNameError, DevicesError, InputCallbackInfo,
    PauseStreamError, PlayStreamError, SampleFormat, SampleRate, StreamConfig, StreamError,
    SupportedBufferSize, SupportedStreamConfigRange, SupportedStreamConfigsError,
};

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
    // {
    //     self.build_input_stream_raw(
    //         config,
    //         T::FORMAT,
    //         move |data, info| {
    //             data_callback(
    //                 data.as_slice()
    //                     .expect("host supplied incorrect sample type"),
    //                 info,
    //             )
    //         },
    //         error_callback,
    //     )
    // }

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
    // {
    //     self.build_output_stream_raw(
    //         config,
    //         T::FORMAT,
    //         move |data, info| {
    //             data_callback(
    //                 data.as_slice_mut()
    //                     .expect("host supplied incorrect sample type"),
    //                 info,
    //             )
    //         },
    //         error_callback,
    //     )
    // }

    // /// Create a dynamically typed input stream.
    // fn build_input_stream_raw<D, E>(
    //     &self,
    //     config: &StreamConfig,
    //     sample_format: SampleFormat,
    //     data_callback: D,
    //     error_callback: E,
    // ) -> Result<Self::Stream, BuildStreamError>
    // where
    //     D: FnMut(&Data, &InputCallbackInfo) + Send + 'static,
    //     E: FnMut(StreamError) + Send + 'static;

    // /// Create a dynamically typed output stream.
    // fn build_output_stream_raw<D, E>(
    //     &self,
    //     config: &StreamConfig,
    //     sample_format: SampleFormat,
    //     data_callback: D,
    //     error_callback: E,
    // ) -> Result<Self::Stream, BuildStreamError>
    // where
    //     D: FnMut(&mut Data, &OutputCallbackInfo) + Send + 'static,
    //     E: FnMut(StreamError) + Send + 'static;
}

#[derive(Default)]
pub struct Devices;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device;

pub struct Host;

#[derive(Debug)]
pub struct Stream {
    audio_thread: Option<JoinHandle<()>>,
    sender: Option<Sender<()>>,
}

impl Drop for Stream {
    #[inline]
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            sender.send(()).unwrap();
        }
        if let Some(thread) = self.audio_thread.take() {
            thread.join().unwrap();
        }
    }
}

pub struct SupportedInputConfigs;
pub struct SupportedOutputConfigs;

impl Host {
    #[allow(dead_code)]
    pub fn new() -> Result<Self, cpal::HostUnavailable> {
        Ok(Host)
    }
}

impl Devices {
    pub fn new() -> Result<Self, DevicesError> {
        Ok(Devices)
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
            channels: 1,
            sample_rate: SampleRate(48000),
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
            channels: 1,
            sample_rate: SampleRate(48000),
            buffer_size: SupportedBufferSize::Range {
                min: 0,
                max: u32::MAX,
            },
            sample_format: SampleFormat::F32,
        })
    }

    fn build_input_stream<T, D, E>(
        &self,
        config: &StreamConfig,
        mut data_callback: D,
        error_callback: E,
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
        config: &StreamConfig,
        mut data_callback: D,
        error_callback: E,
    ) -> Result<Self::Stream, BuildStreamError>
    where
        T: Sample,
        D: FnMut(&mut [T], &OutputCallbackInfo) + Send + 'static,
        E: FnMut(StreamError) + Send + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || {
            let mut buf = [0f32; 128];

            let mut data =
                unsafe { std::slice::from_raw_parts(buf.as_mut_ptr() as *const T, 128) }.to_owned();
            let info = OutputCallbackInfo {
                timestamp: OutputStreamTimestamp {
                    callback: StreamInstant { secs: 0, nanos: 0 },
                    playback: StreamInstant { secs: 0, nanos: 0 },
                },
            };
            loop {
                if let Ok(()) = receiver.try_recv() {
                    break;
                }
                data_callback(&mut data, &info);
            }
        });

        Ok(Self::Stream {
            audio_thread: Some(handle),
            sender: Some(sender),
        })
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
        Some(Device)
    }

    fn default_output_device(&self) -> Option<Device> {
        Some(Device {})
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
