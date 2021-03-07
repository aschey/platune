use act_zero::*;
use log::info;
use postage::{broadcast::Receiver, prelude::Stream};

use crate::libplayer::PlayerEvent;

use super::player::Player;
pub struct SongQueue {
    songs: Vec<String>,
    position: usize,
    player: Addr<Player>,
}
impl Actor for SongQueue {}

impl SongQueue {
    pub fn new(player: Addr<Player>) -> SongQueue {
        SongQueue {
            songs: vec![],
            position: 0,
            player,
        }
    }

    pub async fn on_ended(&mut self) {
        let should_load_next = call!(self.player.should_load_next()).await.unwrap();
        if should_load_next {
            info!("Queue event: ended. Loading next.");
            self.load_next().await;
        } else {
            info!("Queue event: ended. Not loading next.");
            self.position = 0;
        }
    }

    async fn load_next(&mut self) {
        self.position += 1;
        call!(self.player.on_ended()).await.unwrap();
        if let Some(song) = self.songs.get(self.position + 1) {
            call!(self.player.load(song.to_owned(), None))
                .await
                .unwrap();
        }
    }

    pub async fn set_queue(&mut self, queue: Vec<String>) {
        info!("Priming queue");
        self.position = 0;
        self.songs = queue;

        if self.songs.len() > 0 {
            call!(self
                .player
                .load(self.songs.get(0).unwrap().to_owned(), None))
            .await
            .unwrap();
        }
        if self.songs.len() > 1 {
            call!(self
                .player
                .load(self.songs.get(1).unwrap().to_owned(), None))
            .await
            .unwrap();
        }
    }
}
