mod audio_main;
mod duration_updated_actor;
mod player_actor;
mod song_queue_actor;
mod song_start_actor;
mod state_changed_actor;
use byte_slice_cast::AsSliceOf;
use gst::GstBinExtManual;
use gst::{prelude::*, ClockTime};
use gstreamer as gst;
use gstreamer::{glib, prelude::Cast, Pipeline};
use gstreamer_app as gst_app;
use gstreamer_audio as gst_audio;
use player_actor::{PlayerActor, PlayerCommand};
use song_queue_actor::{QueueItem, SongQueueActor};
use song_start_actor::{SongStartActor, StartSeconds};
use tokio::sync::mpsc::{self, Sender};

use gstreamer_player::{
    Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher, PlayerState,
};
use state_changed_actor::{StateChanged, StateChangedActor};

use std::{
    cell::RefCell,
    fs::{self, File},
    io::{Cursor, Read, Seek, SeekFrom},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[tokio::main]
async fn main() {
    gst::init().unwrap();
    // let uri = "file://c/shared_files/Music/4 Strings/Believe/01 Intro.m4a";
    //let uri = "C:\\shared_files\\Music\\The Mars Volta\\Frances the Mute\\06 Cassandra Gemini.mp3";
    let uri = "file://c/shared_files/Music/Between the Buried and Me/Colors/04 Sun Of Nothing.m4a";
    let main_loop = glib::MainLoop::new(None, false);
    //let s = System::new("blah");

    let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
    //let player = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));
    //let clock = gst::SystemClock::obtain();
    //let loaded = RefCell::new(false);

    let (state_tx, mut state_rx) = mpsc::channel::<StateChanged>(32);
    let (start_tx, mut start_rx) = mpsc::channel::<StartSeconds>(32);
    let (player_tx, mut player_rx) = mpsc::channel::<PlayerCommand>(32);
    let player_tx2 = player_tx.clone();
    //let (duration_tx, mut duration_rx) = mpsc::channel(32);
    // let duration_tx2 = duration_tx1.clone();
    // let player1 = make_player(0, duration_tx1);
    // let player2 = make_player(1, duration_tx2);

    let queue = &SongQueueActor::new();

    tokio::spawn(async move {
        let mut player = PlayerActor::new(state_tx);
        while let Some(msg) = player_rx.recv().await {
            match msg {
                PlayerCommand::Play { id } => {
                    player.play(id);
                }
                PlayerCommand::Pause { id } => {
                    player.pause(id);
                }
                PlayerCommand::SetUri { id, uri } => {
                    player.set_uri(id, uri);
                }
                PlayerCommand::Seek { id, position } => {
                    player.seek(id, position);
                }
            }
        }
    });

    // tokio::spawn(async move {
    //     let mut queue = SongQueueActor::new();
    //     while let Some(msg) = duration_rx.recv().await {
    //         queue.recv_duration(msg);
    //     }
    // });

    tokio::spawn(async move {
        //let p1 = &player1.into();
        let mut state_changed_actor = StateChangedActor::new(start_tx);
        while let Some(msg) = state_rx.recv().await {
            state_changed_actor.handle(msg).await;
        }
    });

    tokio::spawn(async move {
        let mut song_start_actor = SongStartActor::new(player_tx);
        while let Some(msg) = start_rx.recv().await {
            song_start_actor.handle(msg).await;
        }
    });

    player_tx2
        .send(PlayerCommand::SetUri {
            id: 0,
            uri: uri.to_owned(),
        })
        .await
        .unwrap();
    player_tx2
        .send(PlayerCommand::Pause { id: 0 })
        .await
        .unwrap();
    player_tx2
        .send(PlayerCommand::Seek {
            id: 0,
            position: (60 * 10 + 50) * 1e9 as u64,
        })
        .await
        .unwrap();
    //player_tx2.
    // let player_weak = player.downgrade();
    // player.connect_media_info_updated(move |playerRef, info| {
    //     let duration = info.get_duration().unwrap_or_default();
    //     if duration > 0 {
    //         if *loaded.borrow() {
    //             println!("loaded");
    //             return;
    //         }
    //         //let clock_weak = clock.downgrade();
    //         *loaded.borrow_mut() = true;
    //         let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
    //         let player2 = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));
    //         player2.set_uri(
    //             //"file://c/shared_files/Music/4 Strings/Believe/02 Take Me Away (Into The Night).m4a",
    //             "file://c/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a"
    //         );
    //         player2.pause();
    //         // player.connect_duration_changed(move |player, time| {
    //         //     println!("duration {:?}", time);
    //         // });
    //         //addr.do_send(StateChanged {player: player.clone(), state: PlayerState::Paused, song_duration: duration});
    //         //let addr2 = addr.clone();
    //         let tx1 = tx1.clone();
    //         //let player_weak = player.downgrade();

    //         let player = player_weak.upgrade().unwrap();
    //         player.connect_state_changed(move |player, player_state| {

    //             tx1.try_send(StateChanged {player: player.clone(), state: player_state, song_duration: duration}).ok();

    //             // println!("{:?}", player_state);
    //             // let clock = clock_weak.upgrade().unwrap();
    //             // if player_state == PlayerState::Playing {
    //             //     let position = player.get_position().nseconds().unwrap();
    //             //     println!("position {:?}", position);
    //             //     let time = clock.get_time();
    //             //     let nseconds = time.nseconds().unwrap();
    //             //     let new_time = ClockTime::from_nseconds(nseconds - position + duration);

    //             //     let shot_id = clock.new_single_shot_id(new_time).unwrap();

    //             //     //let player_weak = player.downgrade();
    //             //     let player2_weak = player2.downgrade();

    //             //     shot_id
    //             //         .wait_async(move |_, _, _| {
    //             //             let player2 = player2_weak.upgrade().unwrap();
    //             //             //let player = player_weak.upgrade().unwrap();

    //             //             player2.play();
    //             //         })
    //             //         .unwrap();

    //         });
    //         player.play();
    //     }
    // });

    // player.set_uri(uri);
    // // Start player so media data is loaded but don't play yet
    // player.pause();
    //player.seek(ClockTime::from_seconds(60 * 10 + 56));

    main_loop.run();
}

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
