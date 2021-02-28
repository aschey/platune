use act_zero::*;

use crate::player_backend::PlayerBackend;

use super::player::Player;
pub struct SongQueue<T: PlayerBackend + Send + 'static> {
    songs: Vec<String>,
    position: usize,
    player: Addr<Player<T>>,
}
impl<T: PlayerBackend + Send + 'static> Actor for SongQueue<T> {}

impl<T: PlayerBackend + Send + 'static> SongQueue<T> {
    pub fn new(player: Addr<Player<T>>) -> SongQueue<T> {
        SongQueue {
            songs: vec![],
            position: 0,
            player,
        }
    }

    pub async fn set_queue(&mut self, queue: Vec<String>) {
        self.songs = queue;
        call!(self.player.load(self.songs.get(0).unwrap().to_owned()))
            .await
            .unwrap();
        call!(self.player.load(self.songs.get(1).unwrap().to_owned()))
            .await
            .unwrap();
    }
}
