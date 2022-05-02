use crate::mock_output::*;
use crate::settings::Settings;
use assert_matches::*;
use async_trait::async_trait;
use futures::Future;
use pretty_assertions::assert_eq;
use rstest::*;
use rubato::{FftFixedInOut, Resampler};
use std::{env::current_dir, time::Duration};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tokio::sync::broadcast;
use tokio::time::{error::Elapsed, timeout};

use crate::platune_player::{PlatunePlayer, PlayerEvent, PlayerState};

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
impl<T: Clone + Send> TimedFut<Option<T>> for broadcast::Receiver<T> {
    async fn timed_recv(&mut self) -> Option<T> {
        timed_await(self.recv()).await.ok().and_then(|r| r.ok())
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

fn decode_sources(
    paths: Vec<String>,
    out_channels: usize,
    sample_rate_out: usize,
    resample_chunk_size: usize,
) -> Vec<f32> {
    let mut all_samples = vec![];
    let mut cur_sample_rate = 44_100;
    let mut resampler = FftFixedInOut::<f64>::new(
        cur_sample_rate,
        sample_rate_out,
        resample_chunk_size,
        out_channels,
    )
    .unwrap();
    let len = paths.len();

    let mut n_frames = resampler.input_frames_next();
    let mut resampler_index = 0;
    let mut resampler_buf = vec![vec![0.0; n_frames]; out_channels];

    for (i, path) in paths.into_iter().enumerate() {
        let mut file_samples = vec![];
        let is_last = i == len - 1;
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
        let in_channels = track.codec_params.channels.unwrap().count();
        let sample_rate = track.codec_params.sample_rate.unwrap() as usize;

        let track_id = track.id;

        let dec_opts = DecoderOptions::default();
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .expect("unsupported codec");

        while let Ok(packet) = format.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(decoded) => {
                    let mut sample_buf =
                        SampleBuffer::<f64>::new(decoded.capacity() as u64, *decoded.spec());

                    sample_buf.copy_interleaved_ref(decoded);

                    let samples = sample_buf.samples();
                    match (in_channels, out_channels) {
                        (1, 2) => {
                            for sample in samples {
                                file_samples.push(*sample);
                                file_samples.push(*sample);
                            }
                        }
                        (2, 1) => {
                            for chunk in samples.chunks_exact(2) {
                                file_samples.push((chunk[0] + chunk[1]) / 2.0);
                            }
                        }
                        _ => {
                            file_samples.extend_from_slice(samples);
                        }
                    };
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

        let mut trimmed = file_samples.into_iter().skip_while(|s| *s == 0.0);
        let mut chan_index = 0;

        if sample_rate != cur_sample_rate {
            if resampler_index > 0 {
                for chan in &mut resampler_buf {
                    chan[resampler_index..].iter_mut().for_each(|d| *d = 0.0);
                }
                let next_resample = resampler.process(&resampler_buf, None).unwrap();
                for i in 0..next_resample[0].len() {
                    for channel in next_resample.iter() {
                        all_samples.push(channel[i]);
                    }
                }
            }

            cur_sample_rate = sample_rate;
            resampler = FftFixedInOut::<f64>::new(
                cur_sample_rate,
                sample_rate_out,
                resample_chunk_size,
                out_channels,
            )
            .unwrap();
            n_frames = resampler.input_frames_next();
            resampler_index = 0;
            resampler_buf = vec![vec![0.0; n_frames]; out_channels];
        }

        if sample_rate == sample_rate_out {
            let trimmed: Vec<f64> = trimmed.collect();
            all_samples.extend_from_slice(&trimmed);
            continue;
        }

        'outer: loop {
            while resampler_index < n_frames {
                match trimmed.next() {
                    Some(next) => {
                        resampler_buf[chan_index][resampler_index] = next;
                        chan_index = (chan_index + 1) % out_channels;
                        if chan_index == 0 {
                            resampler_index += 1;
                        }
                    }
                    None => {
                        if is_last {
                            for chan in &mut resampler_buf {
                                chan[resampler_index..].iter_mut().for_each(|d| *d = 0.0);
                            }
                            let next_resample = resampler.process(&resampler_buf, None).unwrap();
                            for i in 0..next_resample[0].len() {
                                for channel in next_resample.iter() {
                                    all_samples.push(channel[i]);
                                }
                            }
                        }
                        break 'outer;
                    }
                }
            }

            let next_resample = resampler.process(&resampler_buf, None).unwrap();
            for i in 0..next_resample[0].len() {
                for channel in next_resample.iter() {
                    all_samples.push(channel[i]);
                }
            }
            resampler_index = 0;
        }
    }

    all_samples.into_iter().map(|s| s as f32).collect()
}

#[rstest(num_songs, case(1), case(2), case(3))]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_basic(num_songs: usize) {
    let (player, mut receiver, _) = init_player(num_songs).await;
    for _ in 0..num_songs {
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
    let (player, mut receiver, _) = init_player(num_songs).await;

    for i in 0..num_songs {
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
    let (player, mut receiver, _) = init_player(num_songs).await;
    let seek_time = Duration::from_millis(100);
    for i in 0..num_songs {
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

#[rstest(
    num_songs,
    next_index,
    case(1, 0),
    case(2, 0),
    case(2, 1),
    case(3, 0),
    case(3, 1),
    case(3, 2)
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_next(num_songs: usize, next_index: usize) {
    let (player, mut receiver, _) = init_player(num_songs).await;

    for i in 0..num_songs {
        if i == next_index {
            player.next().await.unwrap();
            if next_index < num_songs - 1 {
                assert_matches!(receiver.timed_recv().await, Some(PlayerEvent::Next(PlayerState {queue_position, ..})) if queue_position == i + 1);
            }
        }
        if next_index == num_songs - 1 || i < num_songs - 1 {
            assert_matches!(receiver.timed_recv().await, Some(PlayerEvent::Ended(_)));
        }
    }

    assert_matches!(
        receiver.timed_recv().await,
        Some(PlayerEvent::QueueEnded(_))
    );
    player.join().await.unwrap();
}

#[rstest(
    num_songs,
    prev_index,
    case(1, 0),
    case(2, 0),
    case(2, 1),
    case(3, 0),
    case(3, 1),
    case(3, 2)
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_prev(num_songs: usize, prev_index: usize) {
    let (player, mut receiver, _) = init_player(num_songs).await;

    for i in 0..num_songs {
        if i == prev_index {
            player.previous().await.unwrap();
            if prev_index > 0 {
                assert_matches!(receiver.timed_recv().await, Some(PlayerEvent::Previous(PlayerState {queue_position, ..})) if queue_position == i - 1);
            }
        }
        assert_matches!(receiver.timed_recv().await, Some(PlayerEvent::Ended(_)));
    }
    if prev_index > 0 {
        assert_matches!(receiver.timed_recv().await, Some(PlayerEvent::Ended(_)));
    }

    assert_matches!(
        receiver.timed_recv().await,
        Some(PlayerEvent::QueueEnded(_))
    );
    player.join().await.unwrap();
}

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_decode_all_data(
    #[values("test_44100.mp3", "test.mp3", "test_stereo_44100.mp3")] source_1: &str,
    #[values(
        Some("test_44100.mp3"),
        Some("test.mp3"),
        Some("test_stereo_44100.mp3"),
        None
    )]
    source_2: Option<&str>,
    #[values(1, 2)] out_channels: usize,
    #[values(44_100, 48_000)] sample_rate_out: u32,
    #[values(1024, 666)] resample_chunk_size: usize,
) {
    let host = Host::new_with_options(
        Duration::from_millis(1),
        sample_rate_out,
        out_channels as u16,
    )
    .unwrap();
    let mut data_rx = host.default_output_device().unwrap().subscribe_data();

    let player = PlatunePlayer::new_with_host(
        Settings {
            enable_resampling: true,
            resample_chunk_size,
        },
        host,
    );

    let mut sources = vec![source_1];
    if let Some(source_2) = source_2 {
        sources.push(source_2);
    }

    let paths: Vec<String> = sources.into_iter().map(get_path).collect();

    let expected_data = decode_sources(
        paths.clone(),
        out_channels,
        sample_rate_out as usize,
        resample_chunk_size,
    );

    player.set_queue(paths).await.unwrap();
    let mut all_data: Vec<f32> = vec![];
    let mut started = false;

    while all_data.len() < expected_data.len() {
        let mut data = data_rx.timed_recv().await.unwrap();
        if !started {
            data = data.into_iter().skip_while(|d| *d == 0.0).collect();
            if !data.is_empty() {
                started = true;
            }
        }
        all_data.extend_from_slice(&data);
    }

    for (i, d) in all_data.iter().enumerate() {
        if i >= expected_data.len() {
            assert_eq!(0.0, *d, "failed at index {i}");
        } else {
            assert_eq!(expected_data[i], *d, "failed at index {i}");
        }
    }
}

#[rstest(wait_for_recv, case(true), case(false))]
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_decode_invalid(wait_for_recv: bool) {
    let player = PlatunePlayer::new(Default::default());
    let mut receiver = player.subscribe();
    player
        .set_queue(vec![get_path("invalid_file.mp3")])
        .await
        .unwrap();
    if wait_for_recv {
        receiver.timed_recv().await;
    }

    assert_matches!(timed_await(player.join()).await, Ok(Ok(())));
}
