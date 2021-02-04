use std::collections::VecDeque;

use gstreamer::prelude::Cast;
use gstreamer_player::{
    Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher, PlayerState,
};

pub struct SongQueueActor {
    //pub players: [Player; 2],
    pub songs: VecDeque<QueueItem>,
    count: usize,
    current_state: PlayerState,
}

impl SongQueueActor {
    pub fn new() -> SongQueueActor {
        // let player1 = SongQueueActor::make_player(self.to_owned());
        // let player2 = SongQueueActor::make_player();
        SongQueueActor {
            //players: [player1, player2],
            songs: VecDeque::new(),
            count: 0,
            current_state: PlayerState::Stopped,
        }
    }

    pub fn enqueue(&mut self, uri: String) {
        self.songs.push_back(QueueItem { uri, duration: 0 });
    }

    pub fn start(&mut self, player: Player) {
        let cur_song = self.songs.pop_front().unwrap();
        player.set_uri(&cur_song.uri);
        player.pause();
    }

    // pub fn recv_duration(&mut self, item: QueueItem) {
    //     let mut song = self.songs.iter().find(|s| s.uri == item.uri).unwrap();
    //     song.duration = item.duration;
    // }

    // fn make_player(queue: &SongQueueActor) -> Player {
    //     let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
    //     let player = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));

    //     player.connect_media_info_updated(move |player, info| {
    //         queue.recv_duration(info.get_duration().unwrap());
    //     });

    //     player.connect_state_changed(move |player, player_state| {
    //         // send(player_state)
    //     });

    //     return player;
    // }
}
#[derive(Clone)]
pub struct QueueItem {
    uri: String,
    duration: u64,
    //player: Player,
}
