use stream_download::registry::Input;

use super::audio_status::AudioStatus;

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub volume: f32,
    pub(crate) queue: Vec<Input>,
    pub queue_position: usize,
    pub status: AudioStatus,
}

impl PlayerState {
    pub fn queue(&self) -> Vec<String> {
        self.queue.iter().map(|q| q.to_string()).collect()
    }
}
