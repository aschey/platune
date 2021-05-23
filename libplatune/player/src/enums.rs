use strum_macros::Display;
#[derive(Debug, Clone)]
pub enum Command {
    SetQueue(Vec<String>),
    Seek(u64),
    SetVolume(f32),
    Pause,
    Resume,
    Start,
    Stop,
    Ended,
    Next,
    Previous,
    Shutdown,
}

#[derive(Clone, Debug, Display)]
pub enum PlayerEvent {
    StartQueue { queue: Vec<String> },
    Stop,
    Pause,
    Resume,
    Ended,
    Next,
    Previous,
    SetVolume { volume: f32 },
    Seek { millis: u64 },
    QueueEnded,
}
