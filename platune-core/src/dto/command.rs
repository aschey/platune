use std::{fmt::Debug, time::Duration};

use super::player_status::TrackStatus;

#[derive(Clone)]
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
    Shutdown,
}

impl Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetQueue(arg0) => f.debug_tuple("SetQueue").field(arg0).finish(),
            Self::AddToQueue(arg0) => f.debug_tuple("AddToQueue").field(arg0).finish(),
            Self::Seek(arg0) => f.debug_tuple("Seek").field(arg0).finish(),
            Self::SetVolume(arg0) => f.debug_tuple("SetVolume").field(arg0).finish(),
            Self::Pause => write!(f, "Pause"),
            Self::Resume => write!(f, "Resume"),
            Self::GetCurrentStatus => f
                .debug_tuple("GetCurrentStatus")
                .field(&"channel".to_owned())
                .finish(),
            Self::Stop => write!(f, "Stop"),
            Self::Ended => write!(f, "Ended"),
            Self::Next => write!(f, "Next"),
            Self::Previous => write!(f, "Previous"),
            Self::Shutdown => write!(f, "Shutdown"),
        }
    }
}
