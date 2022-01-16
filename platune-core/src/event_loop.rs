use dasp::{
    interpolate::{linear::Linear, sinc::Sinc},
    ring_buffer::Fixed,
    signal::{
        from_interleaved_samples_iter, interpolate::Converter, FromInterleavedSamplesIterator,
        UntilExhausted,
    },
    Frame, Signal,
};

use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{
    audio_processor::{AudioProcessor, AudioProcessorState},
    dto::{command::Command, player_event::PlayerEvent, player_status::TrackStatus},
    output::{AudioOutput, CpalAudioOutput},
    player::Player,
    source::Source,
    TwoWayReceiverAsync, TwoWaySender,
};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use std::fmt::Debug;
use symphonia::core::units::TimeStamp;
use tokio::sync::broadcast;
use tracing::{error, info};

#[derive(Clone, Debug)]
pub(crate) enum DecoderResponse {
    SeekResponse(Option<TimeStamp>),
    CurrentTimeResponse(CurrentTime),
}

#[derive(Clone, Debug)]
pub(crate) enum PlayerResponse {
    StatusResponse(TrackStatus),
}

#[derive(Clone)]
pub(crate) enum DecoderCommand {
    Seek(Duration),
    Pause,
    Play,
    Stop,
    SetVolume(f64),
    GetCurrentTime,
}

impl Debug for DecoderCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seek(arg0) => f
                .debug_tuple("Seek")
                .field(arg0)
                .field(&"sender".to_owned())
                .finish(),
            Self::Pause => write!(f, "Pause"),
            Self::Play => write!(f, "Play"),
            Self::Stop => write!(f, "Stop"),
            Self::SetVolume(arg0) => f.debug_tuple("SetVolume").field(arg0).finish(),
            Self::GetCurrentTime => f
                .debug_tuple("GetCurrentTime")
                .field(&"channel".to_owned())
                .finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CurrentTime {
    pub current_time: Option<Duration>,
    pub retrieval_time: Option<Duration>,
}

enum InterpolateMode {
    Linear,
    Sinc,
    None,
}

pub(crate) fn decode_loop(
    queue_rx: Receiver<Box<dyn Source>>,
    processor_state: Rc<RefCell<AudioProcessorState>>,
) {
    let mut output = CpalAudioOutput::try_open().unwrap();
    let interpolate_mode = InterpolateMode::Linear;

    loop {
        match queue_rx.try_recv() {
            Ok(source) => {
                decode_source(
                    source,
                    processor_state.clone(),
                    &interpolate_mode,
                    &mut output,
                );
            }
            Err(TryRecvError::Empty) => {
                // If no pending source, stop the output to preserve cpu
                output.stop();
                match queue_rx.recv() {
                    Ok(source) => {
                        output.resume();
                        decode_source(
                            source,
                            processor_state.clone(),
                            &interpolate_mode,
                            &mut output,
                        );
                    }
                    Err(_) => break,
                };
            }
            Err(TryRecvError::Disconnected) => {
                break;
            }
        }
    }
}

fn decode_source(
    source: Box<dyn Source>,
    processor_state: Rc<RefCell<AudioProcessorState>>,
    interpolate_mode: &InterpolateMode,
    output: &mut Box<dyn AudioOutput>,
) {
    let output_sample_rate = output.sample_rate();
    let output_channels = output.channels();
    let mut processor = AudioProcessor::new(source, output_channels, processor_state);
    match (output_channels, &interpolate_mode) {
        (1, InterpolateMode::Linear) => {
            let left = processor.next().unwrap();
            let right = processor.next().unwrap();
            let resampled = linear_resample(left, right, processor, output_sample_rate);

            output.write_stream(resampled);
        }
        (2, InterpolateMode::Linear) => {
            let left = [processor.next().unwrap(), processor.next().unwrap()];
            let right = [processor.next().unwrap(), processor.next().unwrap()];
            let resampled = linear_resample(left, right, processor, output_sample_rate);
            let stereo_resampled = Box::new(StereoStream::new(resampled));

            output.write_stream(stereo_resampled);
        }
        (1, InterpolateMode::Sinc) => {
            let resampled = sinc_resample::<f64>(processor, output_sample_rate);

            output.write_stream(Box::new(resampled));
        }
        (2, InterpolateMode::Sinc) => {
            let resampled = sinc_resample::<[f64; 2]>(processor, output_sample_rate);

            output.write_stream(Box::new(StereoStream::new(resampled)));
        }
        (_, InterpolateMode::None) => {
            output.write_stream(Box::new(processor));
        }
        _ => {}
    }
}

struct StereoStream {
    inner: Box<dyn Iterator<Item = [f64; 2]>>,
    current: Option<[f64; 2]>,
    position: usize,
}

impl StereoStream {
    fn new(mut inner: Box<dyn Iterator<Item = [f64; 2]>>) -> Self {
        let current = inner.next();
        Self {
            inner,
            current,
            position: 0,
        }
    }
}

impl Iterator for StereoStream {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            Some(current) => {
                let next = current[self.position];
                self.position += 1;
                if self.position == 2 {
                    self.position = 0;
                    self.current = self.inner.next();
                }
                Some(next)
            }
            None => None,
        }
    }
}

fn sinc_resample<T>(
    processor: AudioProcessor,
    output_sample_rate: f64,
) -> Box<UntilExhausted<Converter<FromInterleavedSamplesIterator<AudioProcessor, T>, Sinc<[T; 128]>>>>
where
    T: Frame<Sample = f64>,
{
    let buf = [T::EQUILIBRIUM; 128];
    let source_sample_rate = processor.sample_rate();
    let signal = from_interleaved_samples_iter(processor);
    let ring_buffer = Fixed::from(buf);

    let converter = Sinc::new(ring_buffer);

    let new_signal = signal.from_hz_to_hz(converter, source_sample_rate as f64, output_sample_rate);
    Box::new(new_signal.until_exhausted())
}

fn linear_resample<T>(
    left: T,
    right: T,
    processor: AudioProcessor,
    output_sample_rate: f64,
) -> Box<UntilExhausted<Converter<FromInterleavedSamplesIterator<AudioProcessor, T>, Linear<T>>>>
where
    T: Frame<Sample = f64>,
{
    let source_sample_rate = processor.sample_rate();
    let signal = from_interleaved_samples_iter(processor);

    let converter = Linear::new(left, right);

    let new_signal = signal.from_hz_to_hz(converter, source_sample_rate as f64, output_sample_rate);
    Box::new(new_signal.until_exhausted())
}

pub(crate) async fn main_loop(
    mut receiver: TwoWayReceiverAsync<Command, PlayerResponse>,
    event_tx: broadcast::Sender<PlayerEvent>,
    queue_tx: Sender<Box<dyn Source>>,
    queue_rx: Receiver<Box<dyn Source>>,
    cmd_sender: TwoWaySender<DecoderCommand, DecoderResponse>,
) {
    let mut queue = Player::new(event_tx, queue_tx, queue_rx, cmd_sender);

    while let Some(next_command) = receiver.recv().await {
        info!("Got command {:?}", next_command);
        match next_command {
            Command::SetQueue(songs) => {
                queue.set_queue(songs).await;
            }
            Command::AddToQueue(song) => {
                queue.add_to_queue(song).await;
            }
            Command::Seek(millis) => {
                queue.seek(millis).await;
            }
            Command::SetVolume(volume) => {
                queue.set_volume(volume).await;
            }
            Command::Pause => {
                queue.pause().await;
            }
            Command::Resume => {
                queue.play().await;
            }
            Command::Stop => {
                queue.stop().await;
            }
            Command::Ended => {
                queue.on_ended();
            }
            Command::Next => {
                queue.go_next().await;
            }
            Command::Previous => {
                queue.go_previous().await;
            }
            Command::GetCurrentStatus => {
                let current_status = queue.get_current_status();
                if let Err(e) = receiver.respond(PlayerResponse::StatusResponse(current_status)) {
                    error!("Error sending player status");
                }
            }
            Command::Shutdown => {
                return;
            }
        }
        info!("Completed command");
    }
    info!("Request loop completed");
}
