use async_trait::async_trait;

use futures::Future;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tokio::sync::broadcast::error::RecvError;

use crate::mock_output::*;
use assert_matches::*;
use rstest::*;
use std::{env::current_dir, time::Duration};
use tokio::sync::broadcast;
use tokio::time::{error::Elapsed, timeout};

use crate::platune_player::{PlatunePlayer, PlayerEvent};

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
    format!("{dir}{SEPARATOR}..{SEPARATOR}test_assets{SEPARATOR}{song}")
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

async fn init_player(
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

fn decode_source(path: String) -> Vec<f32> {
    let src = std::fs::File::open(&path).expect("failed to open media");

    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    let meta_opts = MetadataOptions::default();
    let fmt_opts = FormatOptions {
        enable_gapless: true,
        ..FormatOptions::default()
    };
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .expect("unsupported format");
    let mut format = probed.format;
    let track = format.default_track().unwrap();
    let track_id = track.id;

    let dec_opts = DecoderOptions::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .expect("unsupported codec");

    let mut all_samples = vec![];
    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let mut sample_buf =
                    SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());

                sample_buf.copy_interleaved_ref(decoded);

                // The interleaved f32 samples can be accessed as follows.
                let samples = sample_buf.samples();
                all_samples.extend_from_slice(samples);
            }
            Err(Error::IoError(_)) => {
                continue;
            }
            Err(Error::DecodeError(_)) => {
                continue;
            }
            Err(err) => {
                panic!("{}", err);
            }
        }
    }

    all_samples.into_iter().skip_while(|s| *s == 0.0).collect()
}

#[rstest(num_songs, case(1), case(2), case(3))]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_basic(num_songs: usize) {
    let (player, mut receiver, songs) = init_player(num_songs).await;
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
    let (player, mut receiver, songs) = init_player(num_songs).await;

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
    let (player, mut receiver, songs) = init_player(num_songs).await;
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

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_decodes_all_data() {
    let host = Host::new_with_sleep(Duration::from_millis(1)).unwrap();
    let mut data_rx = host.default_output_device().unwrap().subscribe_data();

    let player = PlatunePlayer::new_with_host(Default::default(), host);

    let path = get_path("test_stereo.mp3");
    let expected_data = decode_source(path.clone());
    player.set_queue(vec![path]).await.unwrap();
    let mut all_data: Vec<f32> = vec![];
    let mut started = false;

    while all_data.len() < expected_data.len() {
        match data_rx.recv().await {
            Ok(mut data) => {
                if !started {
                    data = data.into_iter().skip_while(|d| *d == 0.0).collect();
                    if !data.is_empty() {
                        started = true;
                    }
                }
                all_data.extend_from_slice(&data);
            }
            Err(RecvError::Lagged(_)) => panic!("lagged"),
            Err(RecvError::Closed) => {
                println!("closed")
            }
        }
    }

    for (i, d) in expected_data.iter().enumerate() {
        assert_eq!(*d, all_data[i]);
    }
}
