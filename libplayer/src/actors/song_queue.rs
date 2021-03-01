use act_zero::*;
use log::info;

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

    pub async fn set_queue(&mut self, queue: Vec<String>) {
        self.songs = queue;

        call!(self
            .player
            .load(self.songs.get(0).unwrap().to_owned(), None))
        .await
        .unwrap();

        //call!(self.player.seek(30.)).await.unwrap();
        call!(self
            .player
            .load(self.songs.get(1).unwrap().to_owned(), None))
        .await
        .unwrap();
    }
}
