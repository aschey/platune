use super::audio_status::AudioStatus;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct CurrentPosition {
    pub position: Duration,
    pub retrieval_time: Option<Duration>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct TrackStatus {
    pub status: AudioStatus,
    pub current_song: Option<String>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct PlayerStatus {
    pub track_status: TrackStatus,
    pub current_position: Option<CurrentPosition>,
}
