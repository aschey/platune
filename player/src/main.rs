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
use song_queue_actor::{QueueCommand, QueueItem, SongQueueActor};
use song_start_actor::{SongStartActor, SongStartCommand};
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

    let main_loop = glib::MainLoop::new(None, false);

    let (state_tx, mut state_rx) = mpsc::channel::<StateChanged>(32);
    let (start_tx, mut start_rx) = mpsc::channel::<SongStartCommand>(32);
    let (queue_command_tx, mut queue_command_rx) = mpsc::channel::<QueueCommand>(32);
    let (player_tx, mut player_rx) = mpsc::channel::<PlayerCommand>(32);

    let start_tx2 = start_tx.clone();
    let player_tx2 = player_tx.clone();
    let player_tx3 = player_tx2.clone();
    let player_tx4 = player_tx3.clone();

    tokio::spawn(async move {
        let mut player = PlayerActor::new(state_tx, player_tx);
        while let Some(msg) = player_rx.recv().await {
            match msg {
                PlayerCommand::Play { id } => {
                    player.play(id);
                }
                PlayerCommand::PlayIfFirst { id } => {
                    player.play_if_first(id);
                }
                PlayerCommand::Pause { id } => {
                    player.pause(id);
                }
                PlayerCommand::SetUri { id, item } => {
                    player.set_uri(id, item);
                }
                PlayerCommand::Seek { id, position } => {
                    player.seek(id, position);
                }
            }
        }
    });

    tokio::spawn(async move {
        let mut queue = SongQueueActor::new(player_tx4, start_tx2);
        while let Some(msg) = queue_command_rx.recv().await {
            match msg {
                QueueCommand::SetQueue { songs } => {
                    queue.set_queue(songs).await;
                }
            }
        }
    });

    tokio::spawn(async move {
        //let p1 = &player1.into();
        let mut state_changed_actor = StateChangedActor::new(start_tx);
        while let Some(msg) = state_rx.recv().await {
            state_changed_actor.handle(msg).await;
        }
    });

    tokio::spawn(async move {
        let mut song_start_actor = SongStartActor::new(player_tx3);
        while let Some(msg) = start_rx.recv().await {
            match msg {
                SongStartCommand::Schedule {
                    nseconds,
                    player_id,
                } => {
                    song_start_actor.handle(nseconds, player_id).await;
                }
                SongStartCommand::RecvItem { item } => {
                    song_start_actor.recv_queue_item(item);
                }
            }
        }
    });

    queue_command_tx
        .send(QueueCommand::SetQueue {
            songs: vec!["file://c/shared_files/Music/Between the Buried and Me/Colors/04 Sun Of Nothing.m4a".to_owned(),
            "file://c/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a".to_owned()],
        })
        .await
        .unwrap();

    player_tx2
        .send(PlayerCommand::Play { id: 0 })
        .await
        .unwrap();
    player_tx2
        .send(PlayerCommand::Seek {
            id: 0,
            position: (60 * 10 + 58) * 1e9 as u64,
        })
        .await
        .unwrap();

    main_loop.run();
}
