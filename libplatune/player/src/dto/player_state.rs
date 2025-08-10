use super::audio_status::AudioStatus;
use super::track::Metadata;
use crate::resolver::TrackInput;

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub volume: f32,
    pub(crate) queue: Vec<TrackInput>,
    pub metadata: Option<Metadata>,
    pub queue_position: usize,
    pub status: AudioStatus,
}

impl PlayerState {
    pub fn queue(&self) -> Vec<String> {
        self.queue.iter().map(|q| q.input.to_string()).collect()
    }
}
