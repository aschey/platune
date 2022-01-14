use cpal::Sample;
use dasp::{Sample as DaspSample, Signal};
use futures_util::StreamExt;
use std::{
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
    audio::{AudioBufferRef, SampleBuffer},
    codecs::DecoderOptions,
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
    Seek(
        Duration,
        //tokio::sync::oneshot::Sender<symphonia::core::errors::Result<SeekedTo>>,
    ),
    Pause,
    Play,
    Stop,
    SetVolume(f32),
    //tokio::sync::oneshot::Sender<CurrentTime>
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

pub(crate) struct Decoder<'a> {
    cmd_receiver: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_sender: &'a TwoWaySenderAsync<Command, PlayerResponse>,
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn symphonia::core::codecs::Decoder>,
    buf: Vec<f32>,
    buf_len: usize,
    pos: usize,
    track_id: u32,
    silence_skipped: bool,
    time_base: TimeBase,
    timestamp: u64,
    sample_buf: SampleBuffer<f32>,
    volume: f32,
    paused: bool,
}

impl<'a> Decoder<'a> {
    pub(crate) fn new(
        reader: Box<dyn FormatReader>,
        decoder: Box<dyn symphonia::core::codecs::Decoder>,
        buf: Vec<f32>,
        time_base: TimeBase,
        cmd_receiver: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
        player_cmd_sender: &'a TwoWaySenderAsync<Command, PlayerResponse>,
        sample_buf: SampleBuffer<f32>,
        volume: f32,
    ) -> Self {
        // Create a hint to help the format registry guess what format reader is appropriate.
        let track_id = reader.default_track().unwrap().id;
        let buf_len = buf.len();
        Self {
            cmd_receiver,
            player_cmd_sender,
            reader,
            decoder,
            buf,
            time_base,
            track_id,
            pos: 0,
            silence_skipped: false,
            timestamp: 0,
            sample_buf,
            volume,
            buf_len,
            paused: false,
        }
    }

    fn process_output(&mut self, packet: &Packet) {
        // Audio samples must be interleaved for cpal. Interleave the samples in the audio
        // buffer into the sample buffer.
        let decoded = self.decoder.decode(packet).unwrap();
        self.sample_buf.copy_interleaved_ref(decoded);
        // Write all the interleaved samples to the ring buffer.
        let samples = self.sample_buf.samples();

        // if !self.silence_skipped {
        //     if let Some(index) = samples.iter().position(|s| *s != 0) {
        //         info!("Skipped {} silent samples", index);
        //         samples = &samples[index..];
        //         self.silence_skipped = true;
        //     }
        // }
        if samples.len() > self.buf.len() {
            self.buf.clear();
            self.buf.resize(samples.len(), 0.0);
        }

        for (i, sample) in samples.iter().enumerate() {
            self.buf[i] = *sample; //* self.volume.to_i16();
        }
        self.buf_len = samples.len();
    }

    fn process_input(&mut self) -> bool {
        match self.cmd_receiver.try_recv() {
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
                        match self.reader.seek(
                            SeekMode::Coarse,
                            SeekTo::Time {
                                time: Time::new(
                                    time.as_secs(),
                                    time.subsec_nanos() as f64 / nanos_per_sec,
                                ),
                                track_id: Some(self.track_id),
                            },
                        ) {
                            Ok(seeked_to) => {
                                if self
                                    .cmd_receiver
                                    .respond(DecoderResponse::SeekResponse(Some(
                                        seeked_to.actual_ts,
                                    )))
                                    .is_err()
                                {
                                    error!("Unable to send seek result");
                                }
                            }
                            Err(e) => {
                                if self
                                    .cmd_receiver
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
                        self.volume = volume;
                    }
                    DecoderCommand::GetCurrentTime => {
                        let time = self.time_base.calc_time(self.timestamp);
                        let millis = ((time.seconds as f64 + time.frac) * 1000.0) as u64;
                        self.cmd_receiver
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

    fn get_next(&mut self) -> Option<&[f32]> {
        // if self.pos < self.buf_len {
        //     let ret = Some(self.buf[self.pos]);
        //     self.pos += 1;
        //     // if self.pos == self.buf.len() {
        //     //     self.pos = 0;
        //     //     self.buf = vec![];
        //     // }
        //     return ret;
        // }

        if !self.process_input() {
            return None;
        }
        // if self.paused {
        //     return Some(0.0);
        // }

        let packet = loop {
            match self.reader.next_packet() {
                Ok(packet) => {
                    if packet.track_id() == self.track_id {
                        break packet;
                    }
                }
                Err(_) => {
                    self.player_cmd_sender.try_send(Command::Ended).unwrap();
                    return None;
                }
            };
        };

        self.timestamp = packet.pts();

        self.process_output(&packet);
        self.pos = 1;

        //Some(self.buf[0])
        Some(&self.buf[..self.buf_len])
    }
}

impl<'a> Iterator for Decoder<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.buf_len {
            let ret = Some(self.buf[self.pos]);
            self.pos += 1;
            // if self.pos == self.buf.len() {
            //     self.pos = 0;
            //     self.buf = vec![];
            // }
            return ret.map(f32::to_sample);
        }

        if !self.process_input() {
            return None;
        }
        if self.paused {
            return Some(0.0);
        }

        let packet = loop {
            match self.reader.next_packet() {
                Ok(packet) => {
                    if packet.track_id() == self.track_id {
                        break packet;
                    }
                }
                Err(_) => {
                    self.player_cmd_sender.try_send(Command::Ended).unwrap();
                    return None;
                }
            };
        };

        self.timestamp = packet.pts();

        self.process_output(&packet);
        self.pos = 1;

        Some(f32::to_sample(self.buf[0]))
    }
}

pub(crate) fn decode_loop(
    queue_rx: Receiver<Box<dyn Source>>,
    mut cmd_receiver: TwoWayReceiver<DecoderCommand, DecoderResponse>,
    player_cmd_sender: TwoWaySenderAsync<Command, PlayerResponse>,
) {
    let mut output = CpalAudioOutput::try_open().unwrap();
    while let Ok(source) = queue_rx.recv() {
        // Create a hint to help the format registry guess what format reader is appropriate.
        let mut hint = Hint::new();
        if let Some(extension) = source.get_file_ext() {
            hint.with_extension(&extension);
        }
        let mss = MediaSourceStream::new(source.as_media_source(), Default::default());

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .unwrap();

        let mut reader = probed.format;

        let track = reader.default_track().unwrap();
        let track_id = track.id;
        let time_base = track.codec_params.time_base.unwrap();
        let decode_opts = DecoderOptions { verify: true };
        let mut symphonia_decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decode_opts)
            .unwrap();

        let (samples, spec, sample_buf) = loop {
            match reader.next_packet() {
                Ok(packet) => {
                    if packet.track_id() != track_id {
                        continue;
                    }
                    match symphonia_decoder.decode(&packet) {
                        Ok(decoded) => {
                            let duration = decoded.capacity();
                            let spec = *decoded.spec();
                            let mut sample_buf = SampleBuffer::new(duration as u64, spec);
                            sample_buf.copy_interleaved_ref(decoded);
                            let samples = sample_buf.samples().to_owned();

                            break (samples, spec, sample_buf);
                        }
                        Err(e) => {
                            continue;
                        }
                    }
                }
                Err(e) => return,
            }
        };

        let output_sample_rate = output.sample_rate();
        let mut decoder = Decoder::new(
            reader,
            symphonia_decoder,
            samples,
            time_base,
            &mut cmd_receiver,
            &player_cmd_sender,
            sample_buf,
            1.0,
        );

        let mut signal = dasp::signal::from_interleaved_samples_iter(decoder);
        let ring_buffer = dasp::ring_buffer::Fixed::from([[0.0, 0.0]; 100]);
        let l = [0.0; 2];
        let r = [0.0; 2];
        // l[0] = signal.next();
        // l[1] = signal.next();
        // r[0] = signal.next();
        // r[1] = signal.next();
        let sinc = dasp::interpolate::linear::Linear::new(l, r);
        //let sinc = dasp::interpolate::sinc::Sinc::new(ring_buffer);

        let new_signal = signal.from_hz_to_hz(sinc, spec.rate as f64, output_sample_rate);
        let mut buf = vec![0.0; 2048];

        let mut i = 0;

        for frame in new_signal.until_exhausted() {
            if i == buf.len() {
                output.write(buf.as_slice());
                i = 0;
            }

            buf[i] = frame[0].to_sample();
            buf[i + 1] = frame[1].to_sample();

            i += 2;
        }
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
