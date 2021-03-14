use core::fmt;
use flexi_logger::{style, DeferredNow, LogTarget, Logger, Record};
use log::info;
use rstest::*;
use std::{
    env::current_dir,
    thread,
    time::{Duration, Instant},
};
use yansi::{Color, Style};

use assert_matches::*;
use platune_libplayer::libplayer::PlatunePlayer;
use platune_libplayer::libplayer::PlayerEvent;
use postage::{broadcast::Receiver, prelude::Stream};

#[cfg(not(target_os = "windows"))]
pub static SEPARATOR: &str = "/";
#[cfg(target_os = "windows")]
pub static SEPARATOR: &str = "\\";

struct SongInfo {
    path: String,
    name: String,
    duration_estimate_millis: u128,
}

trait SongVec {
    fn get_paths(&self) -> Vec<String>;
}

impl SongVec for Vec<SongInfo> {
    fn get_paths(&self) -> Vec<String> {
        self.iter().map(|s| s.path.to_owned()).collect()
    }
}

fn colored(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> core::result::Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "[{} {}] {} [{}:{}] {}",
        Style::new(Color::Cyan).paint(now.now().format("%Y-%m-%d %H:%M:%S%.6f")),
        Style::new(Color::RGB(119, 102, 204)).paint(now.now().timestamp_nanos() as f64 / 1e9),
        style(level, level),
        Style::new(Color::Green).paint(record.file().unwrap_or("<unnamed>")),
        Style::new(Color::Green).paint(record.line().unwrap_or(0)),
        style(level, &record.args())
    )
}

#[ctor::ctor]
fn init() {
    gstreamer::init().unwrap();
    Logger::with_str("info")
        .format_for_stdout(colored)
        .log_target(LogTarget::StdOut)
        .set_palette("196;190;-;-;-".to_owned())
        .start()
        .unwrap();
}

fn get_path(song: &str) -> String {
    let dir = current_dir().unwrap().to_str().unwrap().to_owned();
    format!("{1}{0}tests{0}assets{0}{2}", SEPARATOR, dir, song).to_string()
}

fn get_test_files(num_songs: usize) -> Vec<SongInfo> {
    let song1 = SongInfo {
        name: "test.mp3".to_owned(),
        path: get_path("test.mp3"),
        duration_estimate_millis: 444,
    };
    let song2 = SongInfo {
        name: "test2.mp3".to_owned(),
        path: get_path("test2.mp3"),
        duration_estimate_millis: 731,
    };
    let song3 = SongInfo {
        name: "test3.mp3".to_owned(),
        path: get_path("test3.mp3"),
        duration_estimate_millis: 731,
    };

    match num_songs {
        1 => vec![song1],
        2 => vec![song1, song2],
        3 => vec![song1, song2, song3],
        _ => vec![],
    }
}

fn assert_duration(min: u128, val: u128) {
    assert!((min - 50) <= val && val < min + 50, "duration={}", val);
}

async fn init_play(num_songs: usize) -> (PlatunePlayer, Receiver<PlayerEvent>, Vec<SongInfo>) {
    let (mut player, mut receiver) = PlatunePlayer::create_dummy();

    let songs = get_test_files(num_songs);
    player.set_queue(songs.get_paths());
    let first = &songs[0];

    assert_matches!(receiver.recv().await, Some(PlayerEvent::Play { file }) if file == first.name);
    (player, receiver, songs)
}

#[rstest(num_songs, case(1), case(2), case(3))]
#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_basic(num_songs: usize) {
    info!("here");
    let (mut player, mut receiver, songs) = init_play(num_songs).await;
    info!("here2");
    for song in songs {
        assert_matches!( receiver.recv().await, Some(PlayerEvent::Ended { file }) if file == song.name);
    }

    assert_matches!(receiver.recv().await, Some(PlayerEvent::QueueEnded));
    player.join();
}

#[rstest(
    num_songs,
    pause_index,
    case(1, 0),
    case(2, 0),
    case(2, 1),
    case(3, 0),
    case(3, 1),
    case(3, 2)
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_pause(num_songs: usize, pause_index: usize) {
    let (mut player, mut receiver, songs) = init_play(num_songs).await;

    for (i, song) in songs.iter().enumerate() {
        if i == pause_index {
            player.pause();
            assert_matches!(receiver.recv().await, Some(PlayerEvent::Pause { file }) if file == song.name);
            player.resume();
            assert_matches!(receiver.recv().await, Some(PlayerEvent::Resume { file }) if file == song.name);
        }
        assert_matches!( receiver.recv().await, Some(PlayerEvent::Ended { file }) if file == song.name);
    }
    assert_matches!(receiver.recv().await, Some(PlayerEvent::QueueEnded));
    player.join();
}

#[rstest(
    num_songs,
    seek_index,
    case(1, 0),
    case(2, 0),
    case(2, 1),
    case(3, 0),
    case(3, 1),
    case(3, 2)
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_seek(num_songs: usize, seek_index: usize) {
    // let num_songs = 1;
    // let seek_index = 0;
    let (mut player, mut receiver, songs) = init_play(num_songs).await;
    let seek_time = 0.1;
    for (i, song) in songs.iter().enumerate() {
        if i == seek_index {
            thread::sleep(Duration::from_millis(1000));
            player.seek(seek_time);
            assert_matches!(receiver.recv().await, Some(PlayerEvent::Seek { file, time }) if file == song.name && time == seek_time);
        }
        assert_matches!( receiver.recv().await, Some(PlayerEvent::Ended { file }) if file == song.name);
    }

    assert_matches!(receiver.recv().await, Some(PlayerEvent::QueueEnded));
    player.join();
}
