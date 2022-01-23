use async_trait::async_trait;

use futures::Future;

use libplatune_player::platune_player::{PlatunePlayer, PlayerEvent};
use rstest::*;
use std::{env::current_dir, time::Duration};
use tokio::sync::broadcast;
use tokio::time::{error::Elapsed, timeout};

use assert_matches::*;

#[cfg(not(target_os = "windows"))]
static SEPARATOR: &str = "/";
#[cfg(target_os = "windows")]
static SEPARATOR: &str = "\\";

struct SongInfo {
    path: String,
}

trait SongVec {
    fn get_paths(&self) -> Vec<String>;
}

#[async_trait]
trait TimedFut<T> {
    async fn timed_recv(&mut self) -> T;
}

#[async_trait]
impl TimedFut<Option<PlayerEvent>> for broadcast::Receiver<PlayerEvent> {
    async fn timed_recv(&mut self) -> Option<PlayerEvent> {
        timed_await(self.recv()).await.unwrap().ok()
    }
}
impl SongVec for Vec<SongInfo> {
    fn get_paths(&self) -> Vec<String> {
        self.iter().map(|s| s.path.to_owned()).collect()
    }
}

#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt()
        .pretty()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_test_writer()
        .init();
}

fn get_path(song: &str) -> String {
    let dir = current_dir().unwrap().to_str().unwrap().to_owned();
    format!("{1}{0}tests{0}assets{0}{2}", SEPARATOR, dir, song)
}

fn get_test_files(num_songs: usize) -> Vec<SongInfo> {
    let song1 = SongInfo {
        path: get_path("test.mp3"),
    };
    let song2 = SongInfo {
        path: get_path("test2.mp3"),
    };
    let song3 = SongInfo {
        path: get_path("test3.mp3"),
    };

    match num_songs {
        1 => vec![song1],
        2 => vec![song1, song2],
        3 => vec![song1, song2, song3],
        _ => vec![],
    }
}

async fn timed_await<T>(future: T) -> Result<T::Output, Elapsed>
where
    T: Future,
{
    timeout(Duration::from_secs(10), future).await
}

async fn init_play(
    num_songs: usize,
) -> (
    PlatunePlayer,
    broadcast::Receiver<PlayerEvent>,
    Vec<SongInfo>,
) {
    let player = PlatunePlayer::new(Default::default());
    let mut receiver = player.subscribe();

    let songs = get_test_files(num_songs);
    let song_queue = songs.get_paths();
    player.set_queue(song_queue.clone()).await.unwrap();

    assert_matches!(
        receiver.timed_recv().await,
        Some(PlayerEvent::StartQueue(state)) if state.queue == song_queue
    );
    (player, receiver, songs)
}

#[rstest(num_songs, case(1), case(2), case(3))]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_basic(num_songs: usize) {
    let (player, mut receiver, songs) = init_play(num_songs).await;
    for _ in songs {
        assert_matches!(receiver.timed_recv().await, Some(PlayerEvent::Ended(_)));
    }

    assert_matches!(
        timed_await(receiver.recv()).await.unwrap(),
        Ok(PlayerEvent::QueueEnded(_))
    );
    player.join().await.unwrap();
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
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_pause(num_songs: usize, pause_index: usize) {
    let (player, mut receiver, songs) = init_play(num_songs).await;

    for (i, _) in songs.iter().enumerate() {
        if i == pause_index {
            player.pause().await.unwrap();
            assert_matches!(receiver.timed_recv().await, Some(PlayerEvent::Pause(_)));
            player.resume().await.unwrap();
            assert_matches!(receiver.timed_recv().await, Some(PlayerEvent::Resume(_)));
        }
        assert_matches!(receiver.timed_recv().await, Some(PlayerEvent::Ended(_)));
    }
    assert_matches!(
        receiver.timed_recv().await,
        Some(PlayerEvent::QueueEnded(_))
    );
    player.join().await.unwrap();
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
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_seek(num_songs: usize, seek_index: usize) {
    let (player, mut receiver, songs) = init_play(num_songs).await;
    let seek_time = Duration::from_millis(100);
    for (i, _) in songs.iter().enumerate() {
        if i == seek_index {
            player.seek(seek_time).await.unwrap();
            assert_matches!(receiver.timed_recv().await, Some(PlayerEvent::Seek(_,millis)) if millis == seek_time);
        }
        assert_matches!(receiver.timed_recv().await, Some(PlayerEvent::Ended(_)));
    }

    assert_matches!(
        receiver.timed_recv().await,
        Some(PlayerEvent::QueueEnded(_))
    );
    player.join().await.unwrap();
}
