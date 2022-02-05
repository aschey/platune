use std::time::Duration;

#[derive(Clone, Debug)]
pub(crate) enum DecoderCommand {
    WaitForInitialization,
    Seek(Duration),
    Pause,
    Play,
    Stop,
    SetVolume(f64),
    GetCurrentPosition,
}
