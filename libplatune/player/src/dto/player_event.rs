use super::{player_state::PlayerState, player_status::CurrentPosition};
use std::time::Duration;
use strum::Display;

#[derive(Clone, Debug, Display)]
#[cfg_attr(feature = "ffi", derive(uniffi::Enum))]
pub enum PlayerEvent {
    StartQueue { state: PlayerState },
    QueueUpdated { state: PlayerState },
    Stop { state: PlayerState },
    Pause { state: PlayerState },
    Resume { state: PlayerState },
    Ended { state: PlayerState },
    Next { state: PlayerState },
    Previous { state: PlayerState },
    SetVolume { state: PlayerState },
    Seek { state: PlayerState, time: Duration },
    QueueEnded { state: PlayerState },
    Position { current_position: CurrentPosition },
}
