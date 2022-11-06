use std::{fmt::Debug, time::Duration};

#[derive(Clone, Debug)]
pub(crate) enum Command {
    SetQueue(Vec<String>),
    AddToQueue(Vec<String>),
    Seek(Duration),
    SetVolume(f64),
    Pause,
    Resume,
    GetCurrentStatus,
    Stop,
    Ended,
    Next,
    Previous,
    Reset,
}
