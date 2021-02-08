use gstreamer::ClockTime;

use crate::player_backend::{FnMediaInfo, FnPlayerState, PlayerBackend, PlayerInit};

pub struct DummyPlayer {}

impl PlayerInit for DummyPlayer {
    fn init() -> Box<dyn PlayerBackend + Send> {
        Box::new(DummyPlayer {})
    }
}

impl PlayerBackend for DummyPlayer {
    fn play(&self) {
        todo!()
    }

    fn pause(&self) {
        todo!()
    }

    fn set_uri(&self, uri: &str) {
        todo!()
    }

    fn get_position(&self) -> ClockTime {
        todo!()
    }

    fn get_duration(&self) -> ClockTime {
        todo!()
    }

    fn seek(&self, position: ClockTime) {
        todo!()
    }

    fn connect_media_info_updated(&self, f: FnMediaInfo) -> gstreamer::glib::SignalHandlerId {
        todo!()
    }

    fn connect_state_changed(&self, f: FnPlayerState) -> gstreamer::glib::SignalHandlerId {
        todo!()
    }
}
