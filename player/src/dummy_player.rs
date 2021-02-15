use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Mutex,
};

use gstreamer::{
    glib::{translate::FromGlib, SignalHandlerId},
    ClockTime, State,
};
use gstreamer_player::PlayerState;

use crate::player_backend::{FnMediaInfo, FnPlayerState, PlayerBackend};

#[derive(Clone)]
pub struct DummyPlayer<'a> {
    pub actions: &'a Mutex<Vec<PlayerAction>>,
    pub current_uri: String,
}

pub enum PlayerAction {
    SetUri { uri: String },
    Played { uri: String },
    Paused { uri: String },
}

impl<'a> PlayerBackend for DummyPlayer<'a> {
    fn play(&self) {
        &self.actions.lock().unwrap().push(PlayerAction::Played {
            uri: self.current_uri.to_owned(),
        });
        println!("play");
    }

    fn pause(&self) {
        &self.actions.lock().unwrap().push(PlayerAction::Paused {
            uri: self.current_uri.to_owned(),
        });
        println!("pause");
    }

    fn set_uri(&mut self, uri: &str) {
        &self.actions.lock().unwrap().push(PlayerAction::SetUri {
            uri: uri.to_owned(),
        });

        self.current_uri = uri.to_owned();
        println!("set uri");
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
