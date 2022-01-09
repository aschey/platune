use std::{
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    dto::{command::Command, player_event::PlayerEvent},
    output::CpalAudioOutput,
    player::Player,
    source::Source,
};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use std::fmt::Debug;
use symphonia::core::{
    codecs::DecoderOptions,
    formats::{FormatOptions, SeekMode, SeekTo},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
    units::Time,
};
use tokio::sync::broadcast;
use tracing::{error, info};

pub(crate) enum DecoderCommand {
    Seek(Duration),
    Pause,
    Play,
    Stop,
    SetVolume(f32),
    GetCurrentTime(tokio::sync::oneshot::Sender<CurrentTime>),
}

impl Debug for DecoderCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seek(arg0) => f.debug_tuple("Seek").field(arg0).finish(),
            Self::Pause => write!(f, "Pause"),
            Self::Play => write!(f, "Play"),
            Self::Stop => write!(f, "Stop"),
            Self::SetVolume(arg0) => f.debug_tuple("SetVolume").field(arg0).finish(),
            Self::GetCurrentTime(_) => f
                .debug_tuple("GetCurrentTime")
                .field(&"channel".to_owned())
                .finish(),
        }
    }
}

#[derive(Debug)]
pub struct CurrentTime {
    pub current_time: Option<Duration>,
    pub retrieval_time: Option<Duration>,
}

pub(crate) fn decode_loop(
    queue_rx: Receiver<Box<dyn Source>>,
    cmd_receiver: Receiver<DecoderCommand>,
    player_cmd_sender: tokio::sync::mpsc::Sender<Command>,
) {
    let mut output = CpalAudioOutput::try_open().unwrap();
    let mut paused = false;
    let mut timestamp: u64 = 0;
    while let Ok(source) = queue_rx.recv() {
        info!("Got source {:?}", source);
        output.resume();
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
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decode_opts)
            .unwrap();

        // track duration
        let dur = track
            .codec_params
            .n_frames
            .map(|frames| track.codec_params.start_ts + frames);

        while let Ok(packet) = reader.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    let spec = *decoded.spec();
                    output.init_track(spec, decoded.capacity() as u64);
                    output.write(decoded);
                    break;
                }
                Err(e) => {
                    continue;
                }
            }
        }

        loop {
            match cmd_receiver.try_recv() {
                Ok(command) => {
                    info!("Got decoder command {:?}", command);
                    paused = false;
                    match command {
                        DecoderCommand::Play => {
                            output.resume();
                        }
                        DecoderCommand::Stop => {
                            output.stop();
                            break;
                        }
                        DecoderCommand::Seek(time) => {
                            let nanos_per_sec = 1_000_000_000.0;
                            reader
                                .seek(
                                    SeekMode::Coarse,
                                    SeekTo::Time {
                                        time: Time::new(
                                            time.as_secs(),
                                            time.subsec_nanos() as f64 / nanos_per_sec,
                                        ),
                                        track_id: Some(track_id),
                                    },
                                )
                                .unwrap();
                        }
                        DecoderCommand::Pause => {
                            paused = true;
                            output.stop();
                        }
                        DecoderCommand::SetVolume(volume) => {
                            output.set_volume(volume);
                        }
                        DecoderCommand::GetCurrentTime(sender) => {
                            let time = time_base.calc_time(timestamp);
                            let millis = ((time.seconds as f64 + time.frac) * 1000.0) as u64;
                            sender
                                .send(CurrentTime {
                                    current_time: Some(Duration::from_millis(millis)),
                                    retrieval_time: Some(
                                        SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                                    ),
                                })
                                .unwrap();
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    if paused {
                        sleep(Duration::from_millis(10));
                        continue;
                    }
                    let packet = match reader.next_packet() {
                        Ok(packet) => packet,
                        Err(_) => {
                            player_cmd_sender.try_send(Command::Ended).unwrap();
                            break;
                        }
                    };
                    if packet.track_id() != track_id {
                        continue;
                    }

                    match decoder.decode(&packet) {
                        Ok(decoded) => {
                            timestamp = packet.pts();
                            output.write(decoded);
                        }
                        Err(e) => {
                            continue;
                        }
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    break;
                }
            }
        }
    }
}

pub(crate) async fn main_loop(
    mut receiver: tokio::sync::mpsc::Receiver<Command>,
    event_tx: broadcast::Sender<PlayerEvent>,
    queue_tx: Sender<Box<dyn Source>>,
    queue_rx: Receiver<Box<dyn Source>>,
    cmd_sender: Sender<DecoderCommand>,
) {
    let mut queue = Player::new(event_tx, queue_tx, queue_rx, cmd_sender);

    while let Some(next_command) = receiver.recv().await {
        info!("Got command {:?}", next_command);
        match next_command {
            Command::SetQueue(songs) => {
                queue.set_queue(songs);
            }
            Command::AddToQueue(song) => {
                queue.add_to_queue(song);
            }
            Command::Seek(millis) => {
                queue.seek(millis);
            }
            Command::SetVolume(volume) => {
                queue.set_volume(volume);
            }
            Command::Pause => {
                queue.pause();
            }
            Command::Resume => {
                queue.play();
            }
            Command::Stop => {
                queue.stop();
            }
            Command::Ended => {
                queue.on_ended();
            }
            Command::Next => {
                queue.go_next();
            }
            Command::Previous => {
                queue.go_previous();
            }
            Command::GetCurrentStatus(current_status_tx) => {
                let current_status = queue.get_current_status();
                if let Err(e) = current_status_tx.send(current_status) {
                    error!("Error sending player status {:?}", e);
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
