use std::{
    collections::VecDeque,
    fs::File,
    path::{Path, PathBuf},
    thread::JoinHandle,
};

use symphonia::core::{
    codecs::{Decoder, DecoderOptions},
    formats::{FormatOptions, FormatReader},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};
use tokio::sync::mpsc::{channel, Sender};

pub struct Player {
    queue: Vec<PathBuf>,
    cmd_sender: Sender<Command>,
}

struct TrackDecoder {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
}

enum Command {
    SetQueue(Vec<String>),
    AddToQueue(Vec<String>),
    Seek(u64),
    SetVolume(f32),
    Pause,
    Resume,
    //GetCurrentStatus(Sender<PlayerStatus>),
    Stop,
    Ended,
    Next,
    Previous,
    Shutdown,
}

impl Player {
    pub fn new() -> Player {
        let (tx, rx) = channel(32);
        Player {
            queue: Vec::new(),
            cmd_sender: tx,
        }
    }

    pub fn append<T: AsRef<Path>>(&mut self, path: T) {
        self.queue.push(path.as_ref().to_owned());
        //let handle = std::thread::spawn(move || decode(reader, decoder, track_id));
    }
}

fn new_decoder<T: AsRef<Path>>(path: T) -> TrackDecoder {
    // Create a hint to help the format registry guess what format reader is appropriate.
    let mut hint = Hint::new();
    let path = path.as_ref();
    if let Some(extension) = path.extension() {
        if let Some(extension_str) = extension.to_str() {
            hint.with_extension(extension_str);
        }
    }

    let source = Box::new(File::open(path).unwrap());

    let mss = MediaSourceStream::new(source, Default::default());

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

    TrackDecoder {
        reader,
        decoder,
        track_id,
    }
}

fn decode(path: PathBuf) {
    let mut track_decoder = new_decoder(path);
    loop {
        let packet = track_decoder.reader.next_packet().unwrap();
        if packet.track_id() != track_decoder.track_id {
            continue;
        }

        match track_decoder.decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                // print progress
                //packet.pts();

                // send decoded to output
            }
            Err(e) => {
                continue;
            }
        }
    }
}
