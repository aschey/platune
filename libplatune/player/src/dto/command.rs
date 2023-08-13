use std::fmt::Debug;
use std::time::Duration;

#[derive(Clone, Debug)]
pub(crate) enum Command {
    SetQueue(Vec<String>),
    AddToQueue(Vec<String>),
    Seek(Duration),
    SetVolume(f32),
    SetDeviceName(Option<String>),
    Pause,
    Resume,
    GetCurrentStatus,
    Stop,
    Ended,
    Next,
    Previous,
    Shutdown,
    Reset,
}
