#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
pub enum AudioStatus {
    Playing,
    Paused,
    Stopped,
}
