use super::{audio_status::AudioStatus, current_position::CurrentPosition};

#[derive(Clone, Debug)]
pub struct TrackStatus {
    pub status: AudioStatus,
    pub current_song: Option<String>,
}

#[derive(Clone, Debug)]
pub struct PlayerStatus {
    pub track_status: TrackStatus,
    pub current_position: Option<CurrentPosition>,
}
