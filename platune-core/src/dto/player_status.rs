use std::time::Duration;

use super::audio_status::AudioStatus;

pub struct PlayerStatus {
    pub current_time: Option<Duration>,
    pub retrieval_time: Option<Duration>,
    pub status: AudioStatus,
    pub current_song: Option<String>,
}
