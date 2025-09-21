use std::time::Duration;

use crate::platune_player::SeekMode;

#[derive(Clone, Debug)]
pub(crate) enum DecoderCommand {
    WaitForInitialization,
    Seek(Duration, SeekMode),
    Pause,
    Play,
    Stop,
    SetVolume(f32),
    GetCurrentPosition,
    Reset,
}
