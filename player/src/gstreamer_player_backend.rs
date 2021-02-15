use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use gstreamer::{
    glib::SignalHandlerId,
    prelude::{Cast, ObjectExt, ObjectType},
    Clock, ClockExt, ClockId, ClockTime, ElementExt, ElementExtManual, GstObjectExt,
    GstObjectExtManual, Pipeline, PipelineExt, State, SystemClock,
};
use gstreamer_player::{
    Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher, PlayerState,
};
use lazy_static::__Deref;

use crate::player_backend::{FnMediaInfo, FnPlayerState, PlayerBackend, PlayerInfo};

#[derive(Clone)]
pub struct GstreamerPlayer {
    player: Player,
}

impl GstreamerPlayer {
    pub fn new(base_time: ClockTime) -> GstreamerPlayer {
        let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
        let player = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));
        let pipeline = player.get_pipeline().dynamic_cast::<Pipeline>().unwrap();
        pipeline.set_base_time(base_time);

        GstreamerPlayer { player }
    }
}

fn convert_state(state: State) -> PlayerState {
    match state {
        State::Paused => PlayerState::Paused,
        State::Playing => PlayerState::Playing,
        _ => PlayerState::Stopped,
    }
}

impl PlayerBackend for GstreamerPlayer {
    fn play(&self) {
        println!("play");
        self.player.play();
    }

    fn pause(&self) {
        println!("pause");
        self.player.pause();
    }

    fn set_uri(&mut self, uri: &str) {
        println!("set uri");
        self.player.set_uri(uri);
    }

    fn get_position(&self) -> ClockTime {
        self.player.get_position()
    }

    fn get_duration(&self) -> ClockTime {
        self.player.get_duration()
    }

    fn seek(&self, position: ClockTime) {
        self.player.seek(position);
    }

    fn connect_media_info_updated(&self, f: FnMediaInfo) -> SignalHandlerId {
        println!("media info updated");
        self.player
            .connect_media_info_updated(move |player, media_info| {
                let (res, current, future) =
                    player.get_pipeline().get_state(ClockTime::from_nseconds(0));
                f(
                    media_info.to_owned(),
                    PlayerInfo {
                        duration: player.get_duration(),
                        position: player.get_position(),
                        state: convert_state(current),
                    },
                );
            })
    }

    fn connect_state_changed(&self, f: FnPlayerState) -> SignalHandlerId {
        self.player.connect_state_changed(move |player, state| {
            f(
                state,
                PlayerInfo {
                    duration: player.get_duration(),
                    position: player.get_position(),
                    state,
                },
            );
        })
    }
}
