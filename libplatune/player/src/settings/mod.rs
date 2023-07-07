#[derive(Clone, Debug)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Settings {
    pub enable_resampling: bool,
    pub resample_chunk_size: u32,
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
