#[derive(Clone, Debug)]
pub struct Settings {
    pub enable_resampling: bool,
    pub resample_chunk_size: usize,
}
