#[derive(Clone, Debug)]
pub struct PlayerState {
    pub volume: f32,
    pub queue: Vec<String>,
    pub queue_position: usize,
}
