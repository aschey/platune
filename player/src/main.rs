#[cfg(test)]
mod dummy_player;
mod duration_updated_actor;
mod gstreamer_player_backend;
mod player_actor;
mod player_backend;
mod song_queue_actor;
mod song_start_actor;
mod state_changed_actor;
#[cfg(test)]
mod test;
use flexi_logger::{
    colored_default_format, colored_detailed_format, colored_with_thread, style, DeferredNow,
    Duplicate, LogTarget, Logger, Record,
};
use futures::{
    future::{join_all, Flatten, JoinAll},
    FutureExt,
};
use gst::{Clock, ClockExt, SystemClock};
use gstreamer as gst;
use gstreamer::{glib, prelude::Cast, Pipeline};
use gstreamer_app as gst_app;
use gstreamer_audio as gst_audio;
use log::{info, warn};
use player_actor::{PlayerActor, PlayerCommand};
use player_backend::PlayerBackend;
//use player_backend::PlayerInit;
use song_queue_actor::{QueueCommand, QueueItem, SongQueueActor};
use song_start_actor::{SongStartActor, SongStartCommand};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
};

use gstreamer_player_backend::GstreamerPlayer;
use state_changed_actor::{StateChanged, StateChangedActor};
use yansi::{Color, Style};

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
) -> JoinHandle<()> {
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
    })
}

fn make_song_start_task(
    player_tx_: &Sender<PlayerCommand>,
    mut start_rx: Receiver<SongStartCommand>,
) -> JoinHandle<()> {
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
    })
}

fn make_player_task<T: PlayerBackend + Send + Clone + 'static>(
    player1: T,
    player2: T,
    state_tx_: &Sender<StateChanged>,
    player_tx_: &Sender<PlayerCommand>,
    mut player_rx: Receiver<PlayerCommand>,
) -> JoinHandle<()> {
    let player_tx = player_tx_.clone();
    let state_tx = state_tx_.clone();
    tokio::spawn(async move {
        let mut player = PlayerActor::new(player1, player2, state_tx, player_tx);
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
    })
}

fn make_state_change_task(
    start_tx_: &Sender<SongStartCommand>,
    mut state_rx: Receiver<StateChanged>,
) -> JoinHandle<()> {
    let start_tx = start_tx_.clone();
    tokio::spawn(async move {
        let mut state_changed_actor = StateChangedActor::new(start_tx);
        while let Some(msg) = state_rx.recv().await {
            state_changed_actor.handle(msg).await;
        }
    })
}

pub fn start_tasks<T: PlayerBackend + Send + Clone + 'static>(
    player1: T,
    player2: T,
) -> (
    JoinAll<JoinHandle<()>>,
    Sender<PlayerCommand>,
    Sender<QueueCommand>,
) {
    let (state_tx, state_rx) = mpsc::channel::<StateChanged>(32);
    let (start_tx, start_rx) = mpsc::channel::<SongStartCommand>(32);
    let (queue_tx, queue_rx) = mpsc::channel::<QueueCommand>(32);
    let (player_tx, player_rx) = mpsc::channel::<PlayerCommand>(32);

    (
        join_all(vec![
            make_player_task(player1, player2, &state_tx, &player_tx, player_rx),
            make_queue_task(&player_tx, &start_tx, queue_rx),
            make_state_change_task(&start_tx, state_rx),
            make_song_start_task(&player_tx, start_rx),
        ]),
        player_tx,
        queue_tx,
    )
}

pub fn colored(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "[{}] T[{:?}] {} [{}:{}] {}",
        Style::new(Color::Fixed(196))
            .bold()
            .paint(now.now().format("%Y-%m-%d %H:%M:%S%.6f %:z")),
        style(level, thread::current().name().unwrap_or("<unnamed>")),
        style(level, level),
        record.file().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
        style(level, &record.args())
    )
}

#[tokio::main]
async fn main() {
    let mut logger = Logger::with_str("info")
        .format_for_stdout(colored)
        .log_target(LogTarget::StdOut) // write logs to file
        // print warnings and errors also to the console
        .start()
        .unwrap();

    warn!("test log");
    gst::init().unwrap();
    let clock = SystemClock::obtain();
    let base_time = clock.get_time();

    let main_loop = glib::MainLoop::new(None, false);
    let player1 = GstreamerPlayer::new(base_time);
    let player2 = GstreamerPlayer::new(base_time);
    let (tasks, player_tx, queue_tx) = start_tasks(player1, player2);

    // let song1 =
    //     "file://c/shared_files/Music/Between the Buried and Me/Colors/04 Sun Of Nothing.m4a"
    //         .to_owned();
    // let song2 =
    //     "file://c/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a"
    //         .to_owned();

    // let song1 = "file:///home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/04 Sun of Nothing.m4a".to_owned();
    // let song2 = "file:///home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a".to_owned();

    // let song1 =
    //     "file:///home/aschey/windows/shared_files/Music/4 Strings/Believe/01 Intro.m4a".to_owned();
    // let song2 = "file:///home/aschey/windows/shared_files/Music/4 Strings/Believe/02 Take Me Away (Into The Night).m4a".to_owned();

    let song1 = "file:///home/aschey/windows/shared_files/Music/emoisdead/Peu Etre - Langue Et Civilisation Hardcore (199x)/Peu Etre-17-Track 17.mp3"
        .to_owned();
    let song2 = "file:///home/aschey/windows/shared_files/Music/emoisdead/Peu Etre - Langue Et Civilisation Hardcore (199x)/Peu Etre-18-Track 18.mp3"
        .to_owned();

    queue_tx
        .send(QueueCommand::SetQueue {
            songs: vec![song1, song2],
        })
        .await
        .unwrap();

    //player_tx.send(PlayerCommand::Play { id: 0 }).await.unwrap();
    // player_tx
    //     .send(PlayerCommand::Seek {
    //         id: 0,
    //         position: (60 * 10 + 50) * 1e9 as u64,
    //     })
    //     .await
    //     .unwrap();

    main_loop.run();
}
