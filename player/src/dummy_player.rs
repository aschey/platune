use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use gstreamer::{
    glib::{translate::FromGlib, SignalHandlerId},
    ClockTime,
};

use crate::player_backend::{FnMediaInfo, FnPlayerState, PlayerBackend};

#[derive(Clone)]
pub struct DummyPlayer<'a> {
    pub incr: &'a Arc<AtomicU32>,
}

impl<'a> PlayerBackend for DummyPlayer<'a> {
    fn play(&self) {}

    fn pause(&self) {}

    fn set_uri(&self, uri: &str) {
        println!("incr");
        &self.incr.fetch_add(1, Ordering::SeqCst);
    }

    fn get_position(&self) -> ClockTime {
        ClockTime::from_seconds(5)
    }

    fn get_duration(&self) -> ClockTime {
        ClockTime::from_seconds(5)
    }

    fn seek(&self, position: ClockTime) {}

    fn connect_media_info_updated(&self, f: FnMediaInfo) -> SignalHandlerId {
        SignalHandlerId::from_glib(1)
    }

    fn connect_state_changed(&self, f: FnPlayerState) -> SignalHandlerId {
        SignalHandlerId::from_glib(1)
    }
}
