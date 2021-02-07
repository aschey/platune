use gstreamer::{glib::SignalHandlerId, prelude::Cast, ClockTime};
use gstreamer_player::{Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher};

use crate::player_backend::{FnMediaInfo, FnPlayerState, PlayerBackend, PlayerInfo, PlayerInit};

pub struct GstreamerPlayer {
    player: Player,
}

impl GstreamerPlayer {
    pub fn new(player: Player) -> GstreamerPlayer {
        GstreamerPlayer { player }
    }
}

impl PlayerInit for GstreamerPlayer {
    fn init() -> Box<dyn PlayerBackend + Send> {
        let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
        let player = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));
        let gst_player = GstreamerPlayer::new(player);
        Box::new(gst_player)
    }
}

impl PlayerBackend for GstreamerPlayer {
    fn play(&self) {
        self.player.play();
    }

    fn pause(&self) {
        self.player.pause();
    }

    fn set_uri(&self, uri: &str) {
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
