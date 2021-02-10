use gstreamer::{glib::SignalHandlerId, ClockTime};
use gstreamer_player::{PlayerMediaInfo, PlayerState};

pub type FnMediaInfo = Box<dyn Fn(PlayerMediaInfo) + Send>;
pub type FnPlayerState = Box<dyn Fn(PlayerState, PlayerInfo) + Send>;

pub trait PlayerBackend {
    fn play(&self);
    fn pause(&self);
    fn set_uri(&self, uri: &str);
    fn get_position(&self) -> ClockTime;
    fn get_duration(&self) -> ClockTime;
    fn seek(&self, position: ClockTime);
    fn connect_media_info_updated(&self, f: FnMediaInfo) -> SignalHandlerId;
    fn connect_state_changed(&self, f: FnPlayerState) -> SignalHandlerId;
}

pub struct PlayerInfo {
    pub position: ClockTime,
    pub duration: ClockTime,
}
