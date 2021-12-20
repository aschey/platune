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
use symphonia::core::units::Duration;

pub trait AudioOutput {
    fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()>;
    fn flush(&mut self);
    fn set_sample_buf(&self, spec: SignalSpec, duration: Duration);
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
    cpal::Sample + ConvertibleSample + RawSample + std::marker::Send + 'static
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

        // Select proper playback routine based on sample format.
        match config.sample_format() {
            cpal::SampleFormat::F32 => CpalAudioOutputImpl::<f32>::try_open(&device),
            cpal::SampleFormat::I16 => CpalAudioOutputImpl::<i16>::try_open(&device),
            cpal::SampleFormat::U16 => CpalAudioOutputImpl::<u16>::try_open(&device),
        }
    }
}

struct CpalAudioOutputImpl<T: AudioOutputSample>
where
    T: AudioOutputSample,
{
    ring_buf_producer: rb::Producer<T>,
    sample_buf: Option<SampleBuffer<T>>,
    stream: cpal::Stream,
}

impl<T: AudioOutputSample> CpalAudioOutputImpl<T> {
    pub fn try_open(device: &cpal::Device) -> Result<Box<dyn AudioOutput>> {
        // Output audio stream config.
        let supported_config = device.default_input_config().unwrap();
        let config = cpal::StreamConfig {
            channels: supported_config.channels(),
            sample_rate: cpal::SampleRate(44_100),
            buffer_size: cpal::BufferSize::Default,
        };

        // Instantiate a ring buffer capable of buffering 8K (arbitrarily chosen) samples.
        let ring_buf = SpscRb::new(8 * 1024);
        let (ring_buf_producer, ring_buf_consumer) = (ring_buf.producer(), ring_buf.consumer());

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

        Ok(Box::new(CpalAudioOutputImpl {
            ring_buf_producer,
            sample_buf: None,
            stream,
        }))
    }
}

impl<T: AudioOutputSample> AudioOutput for CpalAudioOutputImpl<T> {
    fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()> {
        // Do nothing if there are no audio frames.
        if decoded.frames() == 0 {
            return Ok(());
        }

        // Audio samples must be interleaved for cpal. Interleave the samples in the audio
        // buffer into the sample buffer.
        if let Some(sample_buf) = self.sample_buf {
            sample_buf.copy_interleaved_ref(decoded);
            // Write all the interleaved samples to the ring buffer.
            let mut samples = sample_buf.samples();

            while let Some(written) = self.ring_buf_producer.write_blocking(samples) {
                samples = &samples[written..];
            }
        }

        Ok(())
    }

    fn flush(&mut self) {
        // Flush is best-effort, ignore the returned result.
        let _ = self.stream.pause();
    }

    fn set_sample_buf(&self, spec: SignalSpec, duration: Duration) {
        self.sample_buf = Some(SampleBuffer::<T>::new(duration, spec));
    }
}

// #[cfg(target_os = "linux")]
// pub fn try_open(spec: SignalSpec, duration: Duration) -> Result<Box<dyn AudioOutput>> {
//     pulseaudio::PulseAudioOutput::try_open(spec, duration)
// }
