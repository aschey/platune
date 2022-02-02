use async_trait::async_trait;

use crate::mock_output::*;
use crate::settings::Settings;
use assert_matches::*;
use futures::Future;
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
impl<T: Clone + Send> TimedFut<Option<T>> for broadcast::Receiver<T> {
    async fn timed_recv(&mut self) -> Option<T> {
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

fn decode_sources(
    paths: Vec<String>,
    out_channels: usize,
    enable_resampling: bool,
    sample_rate_in: usize,
    sample_rate_out: usize,
    resample_chunk_size: usize,
) -> Vec<f32> {
    let mut all_samples = vec![];
    let mut resampler = FftFixedInOut::<f64>::new(
        sample_rate_in,
        sample_rate_out,
        resample_chunk_size,
        out_channels,
    );
    let len = paths.len();

    let n_frames = resampler.nbr_frames_needed();
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

        if !enable_resampling {
            let trimmed: Vec<f64> = trimmed.collect();
            all_samples.extend_from_slice(&trimmed);
            continue;
        }

        let mut resampled = vec![];
        let mut chan_index = 0;
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
                            let next_resample = resampler.process(&resampler_buf).unwrap();
                            for i in 0..next_resample[0].len() {
                                for channel in next_resample.iter() {
                                    resampled.push(channel[i]);
                                }
                            }
                        }
                        break 'outer;
                    }
                }
            }

            let next_resample = resampler.process(&resampler_buf).unwrap(); //resampler_buf.clone();
            for i in 0..next_resample[0].len() {
                for channel in next_resample.iter() {
                    resampled.push(channel[i]);
                }
            }
            resampler_index = 0;
        }
        all_samples.extend_from_slice(&resampled);
    }

    all_samples.into_iter().map(|s| s as f32).collect()
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

#[rstest]
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_decode_all_data(
    #[values(vec!["test_stereo.mp3"], vec!["test_stereo.mp3", "test_stereo.mp3"] )] sources: Vec<
        &str,
    >,
    #[values(1, 2)] out_channels: usize,
    #[values(44_100)] sample_rate_in: u32,
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
            enable_resampling: sample_rate_in != sample_rate_out,
            resample_chunk_size,
        },
        host,
    );

    let paths: Vec<String> = sources.into_iter().map(get_path).collect();

    let expected_data = decode_sources(
        paths.clone(),
        out_channels,
        sample_rate_in != sample_rate_out,
        sample_rate_in as usize,
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
