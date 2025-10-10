mod url;
mod yt_dlp;

use decal::decoder::Source;
use stream_download::registry::Input;
pub(crate) use url::*;
pub(crate) use yt_dlp::*;

use crate::dto::track::Metadata;

#[derive(Debug)]
pub(crate) struct MetadataSource {
    pub(crate) source: Box<dyn Source>,
    pub(crate) metadata: Option<Metadata>,
    pub(crate) has_content_length: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackInput {
    pub input: Input,
    pub metadata: Option<Metadata>,
}

// live streams have fixed transfer rates so we'll limit prefetch to 2 seconds
const PREFETCH_SECONDS: u64 = 5;
const LIVE_PREFETCH_SECONDS: u64 = 2;
// store 512kb of audio data for bounded storage
const TEMP_BUFFER_SIZE: usize = 1024 * 512;

fn bitrate_to_prefetch(mut bitrate: u32, content_length: Option<u64>) -> u64 {
    let prefetch_seconds = if content_length.is_some() {
        PREFETCH_SECONDS
    } else {
        LIVE_PREFETCH_SECONDS
    };
    // If bitrate is > 1000, it was probably incorrectly sent as bits/sec instead of kilobits/sec.
    if bitrate > 1000 {
        bitrate /= 1000;
    }
    // buffer 5 seconds of audio
    // bitrate (in kilobits) / bits per byte * bytes per kilobyte * seconds
    (bitrate / 8 * 1000) as u64 * prefetch_seconds
}
