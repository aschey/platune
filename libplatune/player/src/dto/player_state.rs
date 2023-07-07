#[derive(Clone, Debug)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct PlayerState {
    pub volume: f64,
    pub queue: Vec<String>,
    pub queue_position: u32,
}
