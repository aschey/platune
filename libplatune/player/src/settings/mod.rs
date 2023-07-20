#[derive(Clone, Debug)]
#[cfg_attr(feature = "ffi", derive(uniffi::Record))]
pub struct Settings {
    pub resample_chunk_size: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            resample_chunk_size: 1024,
        }
    }
}
