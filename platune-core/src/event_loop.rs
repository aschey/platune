use cpal::Sample;
use dasp::{Sample as DaspSample, Signal};
use futures_util::StreamExt;
use std::{
    cell::RefCell,
    rc::Rc,
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    dto::{command::Command, player_event::PlayerEvent, player_status::TrackStatus},
    output::{AudioOutput, CpalAudioOutput},
    player::Player,
    source::Source,
    TwoWayReceiver, TwoWayReceiverAsync, TwoWaySender, TwoWaySenderAsync,
};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use std::fmt::Debug;
use symphonia::core::{
    audio::{AudioBufferRef, SampleBuffer, SignalSpec},
    codecs::{Decoder as SymphoniaDecoder, DecoderOptions},
    formats::{FormatOptions, FormatReader, Packet, SeekMode, SeekTo, SeekedTo},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
    units::{Time, TimeBase, TimeStamp},
};
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

pub(crate) struct AudioProcessor {
    state: Rc<RefCell<AudioProcessorState>>,
    decoder: Decoder,
    paused: bool,
}

struct Decoder {
    buf: Vec<f64>,
    sample_buf: SampleBuffer<f64>,
    decoder: Box<dyn SymphoniaDecoder>,
    reader: Box<dyn FormatReader>,
    time_base: TimeBase,
    buf_len: usize,
    spec: SignalSpec,
    position: usize,
    track_id: u32,
    channels: usize,
    timestamp: u64,
}

impl Decoder {
    fn new(source: Box<dyn Source>) -> Self {
        let mut hint = Hint::new();
        if let Some(extension) = source.get_file_ext() {
            hint.with_extension(&extension);
        }
        let mss = MediaSourceStream::new(source.as_media_source(), Default::default());

        let format_opts = FormatOptions {
            enable_gapless: true,
            ..FormatOptions::default()
        };
        let metadata_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .unwrap();

        let mut reader = probed.format;

        let track = reader.default_track().unwrap();

        let time_base = track.codec_params.time_base.unwrap();
        let decode_opts = DecoderOptions { verify: true };
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decode_opts)
            .unwrap();
        let track = reader.default_track().unwrap();
        let track_id = track.id;

        let (buf, spec, sample_buf, position, timestamp) =
            Self::process_first_packet(&mut reader, &mut decoder, track_id);

        Self {
            decoder,
            reader,
            time_base,
            buf_len: buf.len(),
            channels: spec.channels.count(),
            track_id,
            buf,
            sample_buf,
            spec,
            position,
            timestamp,
        }
    }

    fn process_first_packet(
        reader: &mut Box<dyn FormatReader>,
        decoder: &mut Box<dyn SymphoniaDecoder>,
        track_id: u32,
    ) -> (Vec<f64>, SignalSpec, SampleBuffer<f64>, usize, u64) {
        loop {
            match reader.next_packet() {
                Ok(packet) => {
                    if packet.track_id() != track_id {
                        continue;
                    }
                    match decoder.decode(&packet) {
                        Ok(decoded) => {
                            let duration = decoded.capacity();
                            let spec = *decoded.spec();
                            let mut sample_buf = SampleBuffer::<f64>::new(duration as u64, spec);
                            sample_buf.copy_interleaved_ref(decoded);
                            let position: usize;
                            let samples = sample_buf.samples();
                            if let Some(index) = samples.iter().position(|s| *s != 0.0) {
                                info!("Skipped {} silent samples", index);
                                position = index;
                            } else {
                                info!("Skipped {} silent samples", samples.len());
                                continue;
                            }

                            return (samples.to_owned(), spec, sample_buf, position, packet.ts());
                        }
                        Err(e) => {
                            continue;
                        }
                    }
                }
                Err(e) => {}
            }
        }
    }

    fn process_output(&mut self, packet: &Packet) {
        // Audio samples must be interleaved for cpal. Interleave the samples in the audio
        // buffer into the sample buffer.
        let decoded = self.decoder.decode(packet).unwrap();
        self.sample_buf.copy_interleaved_ref(decoded);
        // Write all the interleaved samples to the ring buffer.
        let samples = self.sample_buf.samples();

        if self.channels == 1 {
            if samples.len() * 2 > self.buf.len() {
                self.buf.clear();
                self.buf.resize(samples.len() * 2, 0.0);
            }

            let mut i = 0;
            for sample in samples.iter() {
                self.buf[i] = *sample;
                self.buf[i + 1] = *sample;
                i += 2;
            }
            self.buf_len = samples.len() * 2;
        } else {
            if samples.len() > self.buf.len() {
                self.buf.clear();
                self.buf.resize(samples.len(), 0.0);
            }

            for (i, sample) in samples.iter().enumerate() {
                self.buf[i] = *sample;
            }
            self.buf_len = samples.len();
        }
    }
}

impl Iterator for Decoder {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.buf_len {
            let ret = Some(self.buf[self.position]);
            self.position += 1;
            return ret;
        }

        let packet = loop {
            match self.reader.next_packet() {
                Ok(packet) => {
                    if packet.track_id() == self.track_id {
                        break packet;
                    }
                }
                Err(_) => {
                    return None;
                }
            };
        };

        self.timestamp = packet.ts();

        self.process_output(&packet);
        self.position = 1;

        Some(self.buf[0])
    }
}

impl AudioProcessor {
    fn new(source: Box<dyn Source>, state: Rc<RefCell<AudioProcessorState>>) -> Self {
        let decoder = Decoder::new(source);

        Self {
            decoder,
            state,
            paused: false,
        }
    }

    fn sample_rate(&self) -> u32 {
        self.decoder.spec.rate
    }

    fn process_input(&mut self) -> bool {
        let mut state = self.state.borrow_mut();
        match state.cmd_rx.try_recv() {
            Ok(command) => {
                info!("Got decoder command {:?}", command);

                match command {
                    DecoderCommand::Play => {
                        self.paused = false;
                    }
                    DecoderCommand::Stop => {
                        return false;
                    }
                    DecoderCommand::Seek(time) => {
                        let nanos_per_sec = 1_000_000_000.0;
                        match self.decoder.reader.seek(
                            SeekMode::Coarse,
                            SeekTo::Time {
                                time: Time::new(
                                    time.as_secs(),
                                    time.subsec_nanos() as f64 / nanos_per_sec,
                                ),
                                track_id: Some(self.decoder.track_id),
                            },
                        ) {
                            Ok(seeked_to) => {
                                if state
                                    .cmd_rx
                                    .respond(DecoderResponse::SeekResponse(Some(
                                        seeked_to.actual_ts,
                                    )))
                                    .is_err()
                                {
                                    error!("Unable to send seek result");
                                }
                            }
                            Err(e) => {
                                if state
                                    .cmd_rx
                                    .respond(DecoderResponse::SeekResponse(None))
                                    .is_err()
                                {
                                    error!("Unable to send seek result");
                                }
                            }
                        }
                    }
                    DecoderCommand::Pause => {
                        self.paused = true;
                    }
                    DecoderCommand::SetVolume(volume) => {
                        state.volume = volume;
                    }
                    DecoderCommand::GetCurrentTime => {
                        let time = self.decoder.time_base.calc_time(self.decoder.timestamp);
                        let millis = ((time.seconds as f64 + time.frac) * 1000.0) as u64;
                        state
                            .cmd_rx
                            .respond(DecoderResponse::CurrentTimeResponse(CurrentTime {
                                current_time: Some(Duration::from_millis(millis)),
                                retrieval_time: Some(
                                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                                ),
                            }))
                            .unwrap();
                    }
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {}
        }

        true
    }
}

impl Iterator for AudioProcessor {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.process_input() {
            return None;
        }

        if self.paused {
            return Some(0.0);
        }

        let state = self.state.borrow_mut();
        match self.decoder.next() {
            Some(val) => Some(val * state.volume),
            None => {
                state.player_cmd_tx.try_send(Command::Ended).unwrap();
                None
            }
        }
    }
}

struct AudioProcessorState {
    cmd_rx: TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: TwoWaySenderAsync<Command, PlayerResponse>,
    volume: f64,
}

pub(crate) fn decode_loop(
    queue_rx: Receiver<Box<dyn Source>>,
    cmd_rx: TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_tx: TwoWaySenderAsync<Command, PlayerResponse>,
) {
    let processor_state = Rc::new(RefCell::new(AudioProcessorState {
        cmd_rx,
        player_cmd_tx,
        volume: 1.0,
    }));
    let mut output = CpalAudioOutput::try_open().unwrap();
    let output_sample_rate = output.sample_rate();

    while let Ok(source) = queue_rx.recv() {
        let mut processor = AudioProcessor::new(source, processor_state.clone());
        let l = [processor.next().unwrap(), processor.next().unwrap()];
        let r = [processor.next().unwrap(), processor.next().unwrap()];
        let source_sample_rate = processor.sample_rate();
        let mut signal = dasp::signal::from_interleaved_samples_iter(processor);
        //let ring_buffer = dasp::ring_buffer::Fixed::from([[0.0, 0.0]; 100]);

        let converter = dasp::interpolate::linear::Linear::new(l, r);
        //let converter = dasp::interpolate::sinc::Sinc::new(ring_buffer);

        let new_signal =
            signal.from_hz_to_hz(converter, source_sample_rate as f64, output_sample_rate);
        let until = new_signal.until_exhausted();
        let iter = Box::new(until);

        output.write_all(iter);
    }
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
