use act_zero::*;
use log::info;
use postage::broadcast;

use crate::{libplayer::PlayerEvent, util::get_filename_from_path};

use super::player::{Player, SenderExt};
pub struct SongQueue {
    songs: Vec<String>,
    position: usize,
    player: Addr<Player>,
    event_tx: broadcast::Sender<PlayerEvent>,
}
impl Actor for SongQueue {}

impl SongQueue {
    pub fn new(player: Addr<Player>, event_tx: broadcast::Sender<PlayerEvent>) -> SongQueue {
        SongQueue {
            songs: vec![],
            position: 0,
            player,
            event_tx,
        }
    }

    pub async fn on_ended(&mut self) {
        let should_load_next = call!(self.player.should_load_next()).await.unwrap();
        if should_load_next {
            info!("Queue event: ended. Loading next.");
            self.load_next().await;

            if self.position == self.songs.len() {
                self.event_tx.publish(PlayerEvent::QueueEnded);
            }
        } else {
            info!("Queue event: ended. Not loading next.");
            self.position = 0;
        }
    }

    pub async fn next(&mut self) {
        if self.position == self.songs.len() - 1 {
            info!("Cannot load next. Already at end of queue");
            return;
        }
        self.position += 1;
        self.prime().await;

        self.event_tx.publish(PlayerEvent::Next)
    }

    pub async fn previous(&mut self) {
        if self.position == 0 {
            info!("Cannot load previous. Already at beginning of queue");
            return;
        }
        self.position -= 1;
        self.prime().await;

        self.event_tx.publish(PlayerEvent::Previous);
    }

    async fn load_next(&mut self) {
        self.position += 1;
        call!(self.player.on_ended()).await.unwrap();
        self.load_if_exists(self.position + 1).await;
    }

    pub async fn set_queue(&mut self, queue: Vec<String>) {
        info!("Priming queue");
        self.position = 0;
        self.songs = queue.clone();

        self.prime().await;
        self.event_tx.publish(PlayerEvent::StartQueue { queue });
    }

    async fn prime(&mut self) {
        call!(self.player.reset()).await.unwrap();
        call!(self.player.ensure_resumed()).await.unwrap();
        self.load_if_exists(self.position).await;
        self.load_if_exists(self.position + 1).await;
    }

    async fn load_if_exists(&mut self, position: usize) {
        if let Some(song) = self.songs.get(position) {
            call!(self.player.load(song.to_owned(), None))
                .await
                .unwrap();
        }
    }

    fn current_file_name(&self) -> String {
        get_filename_from_path(&self.songs[self.position])
    }
}
