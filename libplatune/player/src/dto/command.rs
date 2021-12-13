use std::sync::mpsc::Sender;

use super::player_status::PlayerStatus;
#[derive(Debug, Clone)]
pub(crate) enum Command {
    SetQueue(Vec<String>),
    AddToQueue(Vec<String>),
    Seek(u64),
    SetVolume(f32),
    Pause,
    Resume,
    GetCurrentStatus(Sender<PlayerStatus>),
    Stop,
    Ended,
    Next,
    Previous,
    Shutdown,
}
