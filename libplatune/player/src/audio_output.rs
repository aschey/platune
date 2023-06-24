#[cfg(not(test))]
pub(crate) use cpal::{
    default_host,
    traits::{DeviceTrait, HostTrait},
    Device, Host, OutputCallbackInfo, Stream, SupportedStreamConfig,
};

#[cfg(test)]
pub(crate) use crate::mock_output::{
    default_host, Device, DeviceTrait, Host, HostTrait, OutputCallbackInfo, Stream,
    SupportedStreamConfig,
};

pub(crate) use cpal::{
    traits::StreamTrait, BuildStreamError, ChannelCount, DefaultStreamConfigError, DevicesError,
    PlayStreamError, SampleFormat, SampleRate, StreamConfig, StreamError,
    SupportedStreamConfigsError,
};
