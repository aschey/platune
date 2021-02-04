use std::cell::RefCell;

use gstreamer::{
    glib::clone::Downgrade,
    prelude::{Cast, ObjectExt},
    ClockId, ClockTime,
};
use gstreamer_player::{Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher};
use tokio::sync::mpsc::Sender;

use crate::{song_queue_actor::QueueItem, state_changed_actor::StateChanged};

pub struct PlayerActor {
    pub players: [Player; 2],
}

impl PlayerActor {
    pub fn new(state_tx: Sender<StateChanged>) -> PlayerActor {
        let state_tx2 = state_tx.clone();
        PlayerActor {
            players: [
                PlayerActor::make_player(0, state_tx),
                PlayerActor::make_player(1, state_tx2),
            ],
        }
    }
    pub fn play(&mut self, id: usize) {
        self.players[id].play();
    }

    pub fn pause(&mut self, id: usize) {
        self.players[id].pause();
    }

    pub fn set_uri(&mut self, id: usize, uri: String) {
        self.players[id].set_uri(&uri);
    }

    pub fn seek(&mut self, id: usize, position: u64) {
        self.players[id].seek(ClockTime::from_nseconds(position));
    }

    fn make_player(
        id: usize,
        // queue_tx: Sender<QueueItem>,
        state_tx: Sender<StateChanged>,
    ) -> Player {
        let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
        let player = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));

        let loaded = RefCell::new(false);
        player.connect_media_info_updated(move |player, info| {
            // info.get_uri()
            // send(info.get_duration())
            if *loaded.borrow() {
                //println!("loaded {:?}", id);
                return;
            }
            let duration = info.get_duration().nseconds().unwrap_or_default();
            // queue_tx.send(QueueItem {
            //     uri: info.get_uri().to_owned(),
            //     duration
            // });

            if duration > 0 {
                if id == 0 {
                    player.play();
                }

                *loaded.borrow_mut() = true;
            }

            //let state_tx = state_tx.clone();
        });

        player.connect_state_changed(move |player, player_state| {
            println!("{:?} {:?}", id, player_state);
            state_tx
                .try_send(StateChanged {
                    player_id: id,
                    state: player_state,
                    position: player.get_position().nseconds().unwrap_or_default(),
                    song_duration: player.get_duration().nseconds().unwrap_or_default(),
                })
                .ok();
        });

        return player;
    }
}

#[derive(Debug)]
pub enum PlayerCommand {
    Play { id: usize },
    Pause { id: usize },
    SetUri { id: usize, uri: String },
    Seek { id: usize, position: u64 },
}
