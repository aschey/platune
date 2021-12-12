use std::{sync::mpsc::Sender, time::Duration};
use strum::Display;
#[derive(Debug, Clone)]
pub(crate) enum Command {
    SetQueue(Vec<String>),
    AddToQueue(Vec<String>),
    Seek(u64),
    SetVolume(f32),
    Pause,
    Resume,
    GetCurrentTime(Sender<Duration>),
    Stop,
    Ended,
    Next,
    Previous,
    Shutdown,
}

#[derive(Clone, Debug)]
pub struct PlayerState {
    pub volume: f32,
    pub queue: Vec<String>,
    pub queue_position: usize,
}

#[derive(Clone, Debug, Display)]
pub enum PlayerEvent {
    StartQueue(PlayerState),
    QueueUpdated(PlayerState),
    Stop(PlayerState),
    Pause(PlayerState),
    Resume(PlayerState),
    Ended(PlayerState),
    Next(PlayerState),
    Previous(PlayerState),
    SetVolume(PlayerState),
    Seek(PlayerState, u64),
    QueueEnded(PlayerState),
}
