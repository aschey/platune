use std::{fs::File, path::Path};

use symphonia::core::{
    codecs::DecoderOptions,
    formats::{FormatOptions, FormatReader},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

fn main() {
    // Create a hint to help the format registry guess what format reader is appropriate.
    let mut hint = Hint::new();
    let path = Path::new("/home/aschey/code/rodio/music.mp3");

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

    loop {
        let packet = reader.next_packet().unwrap();
        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                // print progress
                packet.pts();

                // send decoded to output
            }
            Err(e) => {
                continue;
            }
        }
    }
}
