use std::time::Duration;

use decal::decoder::CurrentPosition;
use strum::Display;

use super::player_state::PlayerState;

#[derive(Clone, Debug, Display)]
pub enum PlayerEvent {
    StartQueue(PlayerState),
    QueueUpdated(PlayerState),
    Stop(PlayerState),
    Pause(PlayerState),
    Resume(PlayerState),
    TrackChanged(PlayerState),
    SetVolume(PlayerState),
    Seek(PlayerState, Duration),
    QueueEnded(PlayerState),
    Position(CurrentPosition),
}
