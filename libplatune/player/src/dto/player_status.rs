use decal::decoder::CurrentPosition;

use super::audio_status::AudioStatus;
use super::player_state::PlayerState;

#[derive(Clone, Debug)]
pub struct TrackStatus {
    pub status: AudioStatus,
    pub state: PlayerState,
}

#[derive(Clone, Debug)]
pub struct PlayerStatus {
    pub track_status: TrackStatus,
    pub current_position: Option<CurrentPosition>,
}
