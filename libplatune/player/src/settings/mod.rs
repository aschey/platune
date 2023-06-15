#[derive(Clone, Debug)]
pub struct Settings {
    pub enable_resampling: bool,
    pub resample_chunk_size: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enable_resampling: true,
            resample_chunk_size: 1024,
        }
    }
}
