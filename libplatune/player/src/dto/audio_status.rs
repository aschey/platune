#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum AudioStatus {
    Playing,
    Paused,
    Stopped,
}
