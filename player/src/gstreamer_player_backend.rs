use gstreamer::{glib::SignalHandlerId, prelude::Cast, ClockTime};
use gstreamer_player::{Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher};

use crate::player_backend::{FnMediaInfo, FnPlayerState, PlayerBackend, PlayerInfo};

#[derive(Clone)]
pub struct GstreamerPlayer {
    player: Player,
}

impl GstreamerPlayer {
    pub fn new() -> GstreamerPlayer {
        let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
        let player = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));
        GstreamerPlayer { player }
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
            .connect_media_info_updated(move |_, media_info| {
                f(media_info.to_owned());
            })
    }

    fn connect_state_changed(&self, f: FnPlayerState) -> SignalHandlerId {
        self.player.connect_state_changed(move |player, state| {
            f(
                state,
                PlayerInfo {
                    duration: player.get_duration(),
                    position: player.get_position(),
                },
            );
        })
    }
}
