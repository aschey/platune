mod audio_main;
mod dummy_player;
mod duration_updated_actor;
mod gstreamer_player_backend;
mod player_actor;
mod player_backend;
mod song_queue_actor;
mod song_start_actor;
mod state_changed_actor;
use gstreamer as gst;
use gstreamer::{glib, prelude::Cast, Pipeline};
use gstreamer_app as gst_app;
use gstreamer_audio as gst_audio;
use player_actor::{PlayerActor, PlayerCommand};
use player_backend::PlayerInit;
use song_queue_actor::{QueueCommand, QueueItem, SongQueueActor};
use song_start_actor::{SongStartActor, SongStartCommand};
use tokio::sync::mpsc::{self, Receiver, Sender};

use gstreamer_player_backend::GstreamerPlayer;
use state_changed_actor::{StateChanged, StateChangedActor};

use std::{
    cell::RefCell,
    fs::{self, File},
    io::{Cursor, Read, Seek, SeekFrom},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

fn make_queue_task(
    player_tx_: &Sender<PlayerCommand>,
    start_tx_: &Sender<SongStartCommand>,
    mut queue_command_rx: Receiver<QueueCommand>,
) {
    let player_tx = player_tx_.clone();
    let start_tx = start_tx_.clone();
    tokio::spawn(async move {
        let mut queue = SongQueueActor::new(player_tx, start_tx);
        while let Some(msg) = queue_command_rx.recv().await {
            match msg {
                QueueCommand::SetQueue { songs } => {
                    queue.set_queue(songs).await;
                }
            }
        }
    });
}

fn make_song_start_task(
    player_tx_: &Sender<PlayerCommand>,
    mut start_rx: Receiver<SongStartCommand>,
) {
    let player_tx = player_tx_.clone();
    tokio::spawn(async move {
        let mut song_start_actor = SongStartActor::new(player_tx);
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
}

fn make_player_task<T: PlayerInit + Send + 'static>(
    state_tx_: &Sender<StateChanged>,
    player_tx_: &Sender<PlayerCommand>,
    mut player_rx: Receiver<PlayerCommand>,
) {
    let player_tx = player_tx_.clone();
    let state_tx = state_tx_.clone();
    tokio::spawn(async move {
        let mut player = PlayerActor::new::<T>(state_tx, player_tx);
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
}

fn make_state_change_task(
    start_tx_: &Sender<SongStartCommand>,
    mut state_rx: Receiver<StateChanged>,
) {
    let start_tx = start_tx_.clone();
    tokio::spawn(async move {
        let mut state_changed_actor = StateChangedActor::new(start_tx);
        while let Some(msg) = state_rx.recv().await {
            state_changed_actor.handle(msg).await;
        }
    });
}

fn start_tasks<T: PlayerInit + Send + 'static>() -> (Sender<PlayerCommand>, Sender<QueueCommand>) {
    let (state_tx, state_rx) = mpsc::channel::<StateChanged>(32);
    let (start_tx, start_rx) = mpsc::channel::<SongStartCommand>(32);
    let (queue_tx, queue_rx) = mpsc::channel::<QueueCommand>(32);
    let (player_tx, player_rx) = mpsc::channel::<PlayerCommand>(32);

    make_player_task::<T>(&state_tx, &player_tx, player_rx);
    make_queue_task(&player_tx, &start_tx, queue_rx);
    make_state_change_task(&start_tx, state_rx);
    make_song_start_task(&player_tx, start_rx);

    (player_tx, queue_tx)
}

#[tokio::main]
async fn main() {
    gst::init().unwrap();

    let main_loop = glib::MainLoop::new(None, false);
    let (player_tx, queue_tx) = start_tasks::<GstreamerPlayer>();

    queue_tx
        .send(QueueCommand::SetQueue {
            songs: vec!["file://c/shared_files/Music/Between the Buried and Me/Colors/04 Sun Of Nothing.m4a".to_owned(),
            "file://c/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a".to_owned()],
        })
        .await
        .unwrap();

    player_tx.send(PlayerCommand::Play { id: 0 }).await.unwrap();
    player_tx
        .send(PlayerCommand::Seek {
            id: 0,
            position: (60 * 10 + 58) * 1e9 as u64,
        })
        .await
        .unwrap();

    main_loop.run();
}
