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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackInput {
    pub input: Input,
    pub metadata: Option<Metadata>,
}
