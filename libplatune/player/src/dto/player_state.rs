#[derive(Clone, Debug)]
pub struct PlayerState {
    pub volume: f64,
    pub queue: Vec<String>,
    pub queue_position: usize,
}
