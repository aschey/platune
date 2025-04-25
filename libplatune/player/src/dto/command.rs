use std::fmt::Debug;
use std::time::Duration;

use crate::platune_player::SeekMode;

#[derive(Clone, Debug)]
pub(crate) enum Command {
    SetQueue(Vec<String>),
    AddToQueue(Vec<String>),
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
    Shutdown,
    Reset,
}
