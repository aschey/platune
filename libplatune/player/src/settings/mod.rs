#[derive(Clone, Debug)]
pub struct Settings {
    pub resample_chunk_size: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            resample_chunk_size: 1024,
        }
    }
}
