// Symphonia
// Copyright (c) 2019-2021 The Project Symphonia Developers.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Platform-dependant Audio Outputs

use std::result;

use symphonia::core::audio::{AudioBufferRef, RawSample, SampleBuffer, SignalSpec};
use symphonia::core::conv::ConvertibleSample;
use symphonia::core::sample::Sample;
use symphonia::core::units::Duration;

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
pub trait AudioOutputSample:
    cpal::Sample + ConvertibleSample + RawSample + std::marker::Send + 'static
{
}

impl AudioOutputSample for f32 {}
impl AudioOutputSample for i16 {}
impl AudioOutputSample for u16 {}

impl CpalAudioOutput {
    pub fn try_open<T: AudioOutputSample>(
        ring_buf_consumer: Consumer<T>,
    ) -> Result<CpalAudioOutputImpl> {
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

        CpalAudioOutputImpl::try_open(&device, ring_buf_consumer)
    }
}

struct CpalAudioOutputImpl {
    stream: cpal::Stream,
}

impl CpalAudioOutputImpl {
    pub fn try_open<T: Clone + Copy + Sample + cpal::Sample + Send + 'static>(
        device: &cpal::Device,
        ring_buf_consumer: Consumer<T>,
    ) -> Result<CpalAudioOutputImpl> {
        // Output audio stream config.
        let config = device.default_input_config().unwrap();

        let config = cpal::StreamConfig {
            channels: config.channels(),
            sample_rate: cpal::SampleRate(44_100),
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

        Ok(CpalAudioOutputImpl { stream })
    }
}
