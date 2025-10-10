use std::fmt::Debug;
use std::time::Duration;

use super::track::{Metadata, Track};
use crate::platune_player::SeekMode;

#[derive(Clone, Debug)]
pub(crate) enum Command {
    SetQueue(Vec<Track>),
    AddToQueue(Vec<Track>),
    Metadata(Metadata),
    Seek(Duration, SeekMode),
    SetVolume(f32),
    SetDeviceName(Option<String>),
    Pause,
    Resume,
    Toggle,
    GetCurrentStatus,
    Stop,
    Ended,
    Next,
    Previous,
    DecoderFailed,
    Reinitialize,
    Shutdown,
    Reset,
}
