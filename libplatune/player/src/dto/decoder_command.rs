use std::time::Duration;

#[derive(Clone, Debug)]
pub(crate) enum DecoderCommand {
    Seek(Duration),
    Pause,
    Play,
    Stop,
    SetVolume(f64),
    GetCurrentPosition,
}
