use std::cell::RefCell;

use gstreamer::{
    glib::clone::Downgrade,
    prelude::{Cast, ObjectExt},
    ClockId, ClockTime,
};
use gstreamer_player::{PlayerMediaInfo, PlayerState};
use tokio::sync::mpsc::Sender;

use crate::{
    player_backend::{FnMediaInfo, PlayerBackend, PlayerInfo, PlayerInit},
    song_queue_actor::QueueItem,
    state_changed_actor::StateChanged,
};

pub struct PlayerActor {
    pub players: [Box<dyn PlayerBackend + Send>; 2],
    position: usize,
}

impl PlayerActor {
    pub fn new<TInit: PlayerInit + Send + 'static>(
        state_tx: Sender<StateChanged>,
        player_tx: Sender<PlayerCommand>,
    ) -> PlayerActor {
        let state_tx2 = state_tx.clone();
        let player_tx2 = player_tx.clone();
        PlayerActor {
            players: [
                PlayerActor::make_player::<TInit>(0, state_tx, player_tx),
                PlayerActor::make_player::<TInit>(1, state_tx2, player_tx2),
            ],
            position: 0,
        }
    }
    pub fn play(&mut self, id: usize) {
        self.players[id].play();
    }

    pub fn play_if_first(&mut self, id: usize) {
        println!("play if first");
        if self.position == 0 {
            self.players[id].play();
        }
    }

    pub fn pause(&mut self, id: usize) {
        self.players[id].pause();
    }

    pub fn set_uri(&mut self, id: usize, item: QueueItem) {
        println!("set uri");
        self.players[id].set_uri(&item.uri);
        self.players[id].pause();
        self.position = item.position;
    }

    pub fn seek(&mut self, id: usize, position: u64) {
        self.players[id].seek(ClockTime::from_nseconds(position));
    }

    fn make_player<TInit: PlayerInit>(
        id: usize,
        state_tx: Sender<StateChanged>,
        player_tx: Sender<PlayerCommand>,
    ) -> Box<dyn PlayerBackend + Send> {
        let player = TInit::init();

        let loaded = RefCell::new(false);
        let c: FnMediaInfo = Box::new(move |info: PlayerMediaInfo| {
            if *loaded.borrow() {
                println!("loaded {:?}", id);
                return;
            }
            let duration = info.get_duration().nseconds().unwrap_or_default();

            if duration > 0 {
                player_tx.try_send(PlayerCommand::PlayIfFirst { id }).ok();

                *loaded.borrow_mut() = true;
            }
        });

        player.connect_media_info_updated(c);
        let c1 = Box::new(move |player_state: PlayerState, info: PlayerInfo| {
            println!("{:?} {:?}", id, player_state);
            state_tx
                .try_send(StateChanged {
                    player_id: id,
                    state: player_state,
                    position: info.position.nseconds().unwrap_or_default(),
                    song_duration: info.duration.nseconds().unwrap_or_default(),
                })
                .ok();
        });

        player.connect_state_changed(c1);

        return player;
    }
}

#[derive(Debug)]
pub enum PlayerCommand {
    Play { id: usize },
    PlayIfFirst { id: usize },
    Pause { id: usize },
    SetUri { id: usize, item: QueueItem },
    Seek { id: usize, position: u64 },
}
