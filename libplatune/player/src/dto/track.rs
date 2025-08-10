use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub url: String,
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Metadata {
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub song: Option<String>,
    pub track_number: Option<u32>,
    pub duration: Option<Duration>,
}
