use std::{
    io::{Read, Seek},
    path::PathBuf,
    sync::mpsc::{Receiver, Sender, SyncSender},
};

use rb::{Consumer, Producer, RbProducer};
use symphonia::core::{
    audio::{SampleBuffer, SignalSpec},
    codecs::DecoderOptions,
    conv::ConvertibleSample,
    formats::FormatOptions,
    io::{MediaSource, MediaSourceStream},
    meta::MetadataOptions,
    probe::Hint,
    sample::Sample,
    units::Duration,
};
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::{
    dto::{command::Command, player_event::PlayerEvent},
    output::{AudioOutputSample, CpalAudioOutput},
    player::Player,
    source::{FileExt, Source},
};

enum OutputCommand {
    NewTrack(SignalSpec, Duration),
}

pub(crate) fn audio_loop<T: AudioOutputSample>(ring_buf_consumer: Consumer<T>) {
    let output = CpalAudioOutput::try_open(ring_buf_consumer).unwrap();
}

pub(crate) fn decode_loop<T: AudioOutputSample>(
    path_rx: Receiver<Box<dyn Source>>,
    ring_buf_producer: Producer<T>,
) {
    while let Ok(source) = path_rx.recv() {
        // Create a hint to help the format registry guess what format reader is appropriate.
        let mut hint = Hint::new();

        if let Some(extension) = source.get_file_ext() {
            hint.with_extension(&extension);
        }

        let mss = MediaSourceStream::new(source.as_media_source(), Default::default());

        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();

        let mut probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .unwrap();

        let mut reader = probed.format;
        let track = reader.default_track().unwrap();
        let track_id = track.id;
        let decode_opts = DecoderOptions { verify: true };
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decode_opts)
            .unwrap();

        // track duration
        let dur = track
            .codec_params
            .n_frames
            .map(|frames| track.codec_params.start_ts + frames);

        loop {
            let packet = reader.next_packet().unwrap();
            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    let spec = *decoded.spec();
                    let duration = decoded.capacity() as u64;
                    let sample_buf = SampleBuffer::new(duration, spec);
                    // print progress
                    //packet.pts();

                    // Do nothing if there are no audio frames.
                    if decoded.frames() == 0 {
                        continue;
                    }

                    // Audio samples must be interleaved for cpal. Interleave the samples in the audio
                    // buffer into the sample buffer.
                    sample_buf.copy_interleaved_ref(decoded);
                    // Write all the interleaved samples to the ring buffer.
                    let mut samples = sample_buf.samples();

                    while let Some(written) = ring_buf_producer.write_blocking(samples) {
                        samples = &samples[written..];
                    }
                }
                Err(e) => {
                    continue;
                }
            }
        }
    }
}

pub(crate) fn ended_loop(receiver: Receiver<Receiver<()>>, request_tx: SyncSender<Command>) {
    while let Ok(ended_receiver) = receiver.recv() {
        // Strange platform-specific behavior here
        // On Windows, receiver.recv() always returns Ok, but on Linux it returns Err
        // after the first event if the queue is stopped
        ended_receiver.recv().unwrap_or_default();
        if let Err(e) = request_tx.send(Command::Ended) {
            error!("Error sending song ended message {:?}", e);
        }
    }
}

pub(crate) fn main_loop(
    receiver: Receiver<Command>,
    finish_rx: Sender<Receiver<()>>,
    event_tx: broadcast::Sender<PlayerEvent>,
    queue_sender: Sender<Box<dyn Source>>,
) {
    // let (_stream, handle) = match rodio::OutputStream::try_default() {
    //     Ok((stream, handle)) => (stream, handle),
    //     Err(e) => {
    //         error!("Error creating audio output stream {:?}", e);
    //         return;
    //     }
    // };

    let mut queue = Player::new(finish_rx, event_tx, queue_sender);

    while let Ok(next_command) = receiver.recv() {
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
