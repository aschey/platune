use stream_download::registry::Input;

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub volume: f32,
    pub(crate) queue: Vec<Input>,
    pub queue_position: usize,
}

impl PlayerState {
    pub fn queue(&self) -> Vec<String> {
        self.queue.iter().map(|q| q.to_string()).collect()
    }
}
