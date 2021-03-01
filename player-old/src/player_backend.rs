use gstreamer::{
    glib::{SignalHandlerId, WeakRef},
    ClockTime, State,
};
use gstreamer_player::{PlayerMediaInfo, PlayerState};

pub type FnMediaInfo = Box<dyn Fn(PlayerMediaInfo, PlayerInfo) + Send>;
//pub type FnPlayerState = Box<dyn Fn(PlayerState, PlayerInfo) + Send>;
pub type FnPlayerInfo = Box<dyn Fn(PlayerInfo) + Send + Sync>;

pub trait PlayerBackend {
    fn play(&self);
    //fn schedule_play(&self, when: ClockTime);
    fn pause(&self);
    fn set_uri(&mut self, uri: &str);
    fn get_position(&self) -> ClockTime;
    fn get_duration(&self) -> ClockTime;
    fn seek(&self, position: ClockTime);
    fn connect_media_info_updated(&self, f: FnMediaInfo) -> SignalHandlerId;
    fn connect_state_changed(&self, f: FnPlayerInfo) -> SignalHandlerId;
    // fn connect_about_to_finish(&self, f: FnPlayerInfo) -> SignalHandlerId;
}

pub struct PlayerInfo {
    pub position: ClockTime,
    pub duration: ClockTime,
    pub state: PlayerState,
}
