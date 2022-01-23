#[derive(Clone, Debug)]
pub struct Settings {
    pub enable_resampling: bool,
    pub resample_chunk_size: usize,
}
impl Default for Settings {
    fn default() -> Self {
        // WASAPI does not resample so it must be done here
        // Otherwise we can leave it to the OS to perform resampling
        let enable_resampling = cfg!(windows);
        Self {
            enable_resampling,
            resample_chunk_size: 1024,
        }
    }
}
