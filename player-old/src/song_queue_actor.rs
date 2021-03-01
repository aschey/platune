use std::collections::VecDeque;

use gstreamer::prelude::Cast;
use gstreamer_player::{
    Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher, PlayerState,
};
use tokio::sync::mpsc::Sender;

use crate::{player_actor::PlayerCommand, song_start_actor::SongStartCommand};

pub struct SongQueueActor {
    pub songs: Vec<String>,
    index: usize,
    player: Sender<PlayerCommand>,
    song_start: Sender<SongStartCommand>,
}

impl SongQueueActor {
    pub fn new(
        player: Sender<PlayerCommand>,
        song_start: Sender<SongStartCommand>,
    ) -> SongQueueActor {
        SongQueueActor {
            songs: vec![],
            index: 0,
            player,
            song_start,
        }
    }

    pub fn enqueue(&mut self, uri: String) {
        self.songs.push(uri)
    }

    pub async fn set_queue(&mut self, queue: Vec<String>) {
        let first = queue[0].to_owned();

        self.index = 0;
        self.player
            .send(PlayerCommand::SetUri {
                id: 0,
                item: QueueItem {
                    uri: first,
                    position: 0,
                },
            })
            .await
            .unwrap();
        if queue.len() > 1 {
            let second = queue[1].to_owned();
            self.song_start
                .send(SongStartCommand::RecvItem {
                    item: QueueItem {
                        uri: second,
                        position: 1,
                    },
                })
                .await
                .unwrap();
        }
        self.songs = queue;
    }

    pub async fn next(&mut self) {
        self.index += 1;
        let next_song = self.songs[self.index].to_owned();
        self.song_start
            .send(SongStartCommand::RecvItem {
                item: QueueItem {
                    uri: next_song,
                    position: self.index,
                },
            })
            .await
            .unwrap();
    }
}
#[derive(Debug, Clone)]
pub struct QueueItem {
    pub uri: String,
    pub position: usize,
}

#[derive(Debug)]
pub enum QueueCommand {
    SetQueue { songs: Vec<String> },
}
