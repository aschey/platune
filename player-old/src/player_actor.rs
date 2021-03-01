use crate::SYSTEM_CLOCK;
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use generic_array::{arr, ArrayLength, GenericArray};
use gstreamer::{
    glib::clone::Downgrade,
    prelude::{Cast, ObjectExt},
    ClockExt, ClockExtManual, ClockId, ClockTime, State, SystemClock,
};
use gstreamer_player::{PlayerMediaInfo, PlayerState};
use log::info;
use tokio::sync::mpsc::Sender;

use crate::{
    player_backend::{FnMediaInfo, PlayerBackend, PlayerInfo},
    song_queue_actor::QueueItem,
    state_changed_actor::StateChanged,
};

pub struct PlayerActor<T: PlayerBackend + Send + 'static> {
    position: usize,
    pub players: GenericArray<T, generic_array::typenum::U2>,
}

impl<T: PlayerBackend + Send + Clone + 'static> PlayerActor<T> {
    pub fn new(
        player1: T,
        player2: T,
        state_tx: Sender<StateChanged>,
        player_tx: Sender<PlayerCommand>,
    ) -> PlayerActor<T> {
        let state_tx2 = state_tx.clone();
        let player_tx2 = player_tx.clone();
        PlayerActor::make_player(0, &player1, state_tx, player_tx);
        PlayerActor::make_player(1, &player2, state_tx2, player_tx2);
        PlayerActor {
            position: 0,
            players: GenericArray::clone_from_slice(&[player1, player2]),
        }
    }
    pub fn play(&mut self, id: usize) {
        self.players[id].play();
    }

    // pub fn schedule_play(&mut self, id: usize, when: i64) {
    //     self.players[id].schedule_play(ClockTime::from_nseconds(when as u64));
    // }

    pub fn play_if_first(&mut self, id: usize) {
        info!("play if first");
        if self.position == 0 {
            info!("playing first");
            self.players[id].play();
        }
    }

    pub fn pause(&mut self, id: usize) {
        info!("pause {:?}", id);
        self.players[0].pause();
        self.players[1].pause();
    }

    pub fn set_uri(&mut self, id: usize, item: QueueItem) {
        self.players[id].set_uri(&item.uri);
        info!("pause {:?}", id);
        self.players[id].pause();
        self.position = item.position;
    }

    pub fn seek(&mut self, id: usize, position: u64) {
        self.players[id].seek(ClockTime::from_nseconds(position));
    }

    fn make_player(
        id: usize,
        player: &T,
        state_tx: Sender<StateChanged>,
        player_tx: Sender<PlayerCommand>,
    ) {
        let loaded = RefCell::new(false);
        let playing = RefCell::new(false);
        let player_tx2 = player_tx.clone();
        player.connect_media_info_updated(Box::new(move |media_info, player_info| {
            //println!("media {:?}", media_info);
            if *loaded.borrow() {
                return;
            }

            info!("duration {:?}", player_info.duration);

            if !*playing.borrow() {
                player_tx2.try_send(PlayerCommand::PlayIfFirst { id }).ok();
                *playing.borrow_mut() = true;
                return;
            }

            let duration = player_info.duration.nseconds().unwrap_or_default();
            if duration > 0 && player_info.state == PlayerState::Playing {
                state_tx
                    .try_send(StateChanged {
                        player_id: id,
                        state: PlayerState::Playing,
                        position: player_info.position.nseconds().unwrap_or_default() as i64,
                        song_duration: duration as i64,
                    })
                    .ok();
                *loaded.borrow_mut() = true;
            }
        }));

        // player.connect_about_to_finish(Box::new(move |info| {
        //     player_tx
        //         .try_send(PlayerCommand::SchedulePlay {
        //             id: id ^ 1,
        //             when: SYSTEM_CLOCK.get_time().nseconds().unwrap() as i64
        //                 - info.position.nseconds().unwrap() as i64
        //                 + info.duration.nseconds().unwrap() as i64
        //                 - 1000000000,
        //         })
        //         .unwrap();
        // }));

        player.connect_state_changed(Box::new(move |info| {
            info!("{:?} {:?}", id, info.state);

            // state_tx
            //     .try_send(StateChanged {
            //         player_id: id,
            //         state: player_state,
            //         position: info.position.nseconds().unwrap_or_default() as i64,
            //         song_duration: info.duration.nseconds().unwrap_or_default() as i64,
            //     })
            //     .ok();
        }));
    }
}

#[derive(Debug)]
pub enum PlayerCommand {
    Play { id: usize },
    PlayIfFirst { id: usize },
    //SchedulePlay { id: usize, when: i64 },
    Pause { id: usize },
    SetUri { id: usize, item: QueueItem },
    Seek { id: usize, position: u64 },
}
