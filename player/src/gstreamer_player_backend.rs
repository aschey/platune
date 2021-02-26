use crate::time::BASE_TIME;
use crate::SYSTEM_CLOCK;
use crate::{glib::timeout_add_seconds, player_backend::FnPlayerInfo};
use gstreamer::{
    format,
    glib::{self, SignalHandlerId},
    prelude::{Cast, ObjectExt, ObjectType},
    query, BusSyncReply, Clock, ClockExt, ClockExtManual, ClockId, ClockTime, Element, ElementExt,
    ElementExtManual, GstBinExt, GstBinExtManual, GstObjectExt, GstObjectExtManual, Pipeline,
    PipelineExt, State, SystemClock,
};
use gstreamer_player::{
    Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher, PlayerState,
};
use lazy_static::__Deref;
use log::info;

use crate::player_backend::{FnMediaInfo, PlayerBackend, PlayerInfo};

#[derive(Clone)]
pub struct GstreamerPlayer {
    player: Player,
}

impl GstreamerPlayer {
    pub fn new() -> GstreamerPlayer {
        let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
        let player = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));

        let pipeline = player.get_pipeline().dynamic_cast::<Pipeline>().unwrap();
        pipeline.set_delay(ClockTime::from_seconds(1));
        //let sink = pipeline.get_children()[0].get_pads()
        pipeline.set_base_time(*BASE_TIME);
        pipeline.set_start_time(*BASE_TIME);
        pipeline.set_clock(Some(&*SYSTEM_CLOCK)).unwrap();

        GstreamerPlayer { player }
    }

    fn downcast(&self) {
        self.player.downgrade();
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
        info!("play");
        self.player.play();
    }

    // fn schedule_play(&self, when: ClockTime) {
    //     let clock_id = SYSTEM_CLOCK.new_single_shot_id(when).unwrap();
    //     let player = self.player.downgrade();
    //     clock_id
    //         .wait_async(move |_, _, _| {
    //             info!("wait event triggered");
    //             player.upgrade().unwrap().play();
    //         })
    //         .unwrap();
    // }

    fn pause(&self) {
        info!("pause");
        self.player.pause();
        self.player.get_pipeline().set_state(State::Paused).unwrap();
    }

    fn set_uri(&mut self, uri: &str) {
        info!("set uri");
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

    // fn connect_about_to_finish(&self, f: FnPlayerInfo) -> SignalHandlerId {
    //     self.player
    //         .get_pipeline()
    //         .connect("about-to-finish", false, move |values| {
    //             // mark the beginning of a new track, or a new DJ.
    //             let playbin = values[0]
    //                 .get::<Element>()
    //                 .expect("playbin \"audio-tags-changed\" signal values[1]")
    //                 .unwrap();
    //             info!(
    //                 "about to finish {:?} {:?}",
    //                 playbin.query_position::<ClockTime>(),
    //                 playbin.query_duration::<ClockTime>()
    //             );
    //             f(PlayerInfo {
    //                 duration: playbin.query_duration::<ClockTime>().unwrap(),
    //                 position: playbin.query_position::<ClockTime>().unwrap(),
    //                 state: PlayerState::Playing,
    //             });
    //             None
    //         })
    //         .unwrap()
    // }

    fn connect_media_info_updated(&self, f: FnMediaInfo) -> SignalHandlerId {
        info!("media info updated");

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

    fn connect_state_changed(&self, f: FnPlayerInfo) -> SignalHandlerId {
        self.player.connect_state_changed(move |player, state| {
            f(PlayerInfo {
                duration: player.get_duration(),
                position: player.get_position(),
                state,
            });
        })
    }
}
