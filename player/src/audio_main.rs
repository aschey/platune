// use gstreamer::prelude::Cast;
// use gstreamer_player::{Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher};
// use tokio::sync::mpsc::{self, Sender};

// use crate::song_queue_actor::{QueueItem, SongQueueActor};

// async fn start() {
//     let (tx1, mut rx1) = mpsc::channel(32);
//     let tx2 = tx1.clone();
//     let player1 = make_player(0, tx1);
//     let player2 = make_player(1, tx2);

//     tokio::spawn(async move {
//         let mut queue = SongQueueActor::new();
//         while let Some(msg) = rx1.recv().await {
//             queue.recv_duration(msg);
//         }
//     });
// }

// fn make_player(id: u8, tx: Sender<QueueItem>) -> Player {
//     let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
//     let player = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));

//     player.connect_media_info_updated(move |player, info| {
//         // info.get_uri()
//         // send(info.get_duration())
//         tx.send(QueueItem {
//             uri: info.get_uri().to_owned(),
//             duration: info.get_duration().nseconds().unwrap(),
//         });
//     });

//     player.connect_state_changed(move |player, player_state| {
//         // send(player_state)
//     });

//     return player;
// }
