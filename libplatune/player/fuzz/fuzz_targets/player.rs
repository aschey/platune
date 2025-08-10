#![no_main]
use std::env::current_dir;
use std::sync::LazyLock;
use std::time::Duration;

use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use libplatune_player::CpalOutput;
use libplatune_player::platune_player::{PlatunePlayer, SeekMode, Settings, Track};
use tokio::runtime::Runtime;

#[derive(Arbitrary, Debug)]
enum NumSongs {
    Zero,
    One,
    Two,
    Three,
}

#[derive(Arbitrary, Debug)]
enum Input {
    SetQueue(NumSongs),
    AddQueue(NumSongs),
    SetVolume(u8),
    Mute,
    Pause,
    Stop,
    Next,
    Previous,
    Seek(u16),
    Resume,
}

static PLAYER: LazyLock<PlatunePlayer<CpalOutput>> =
    LazyLock::new(|| PlatunePlayer::new(CpalOutput::default(), Settings::default()));

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());

fn get_path(song: &str) -> Track {
    let dir = current_dir().unwrap().to_str().unwrap().to_owned();
    let path = format!("{dir}/../../test_assets/{song}");
    Track {
        url: path,
        metadata: None,
    }
}

fn get_test_files(num_songs: NumSongs) -> Vec<Track> {
    let song1 = get_path("test.mp3");
    let song2 = get_path("test2.mp3");
    let song3 = get_path("test3.mp3");

    match num_songs {
        NumSongs::Zero => vec![],
        NumSongs::One => vec![song1],
        NumSongs::Two => vec![song1, song2],
        NumSongs::Three => vec![song1, song2, song3],
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

fuzz_target!(|input: Input| {
    RUNTIME.block_on(async {
        match input {
            Input::SetQueue(num_songs) => {
                PLAYER.set_queue(get_test_files(num_songs)).await.unwrap();
            }
            Input::AddQueue(num_songs) => {
                PLAYER
                    .add_to_queue(get_test_files(num_songs))
                    .await
                    .unwrap();
            }
            Input::SetVolume(volume) => {
                PLAYER
                    .set_volume(((volume as f32) / 255.0).max(0.1))
                    .await
                    .unwrap();
            }
            Input::Mute => {
                PLAYER.set_volume(0.0).await.unwrap();
            }
            Input::Pause => {
                PLAYER.pause().await.unwrap();
            }
            Input::Stop => {
                PLAYER.stop().await.unwrap();
            }
            Input::Next => {
                PLAYER.next().await.unwrap();
            }
            Input::Previous => {
                PLAYER.previous().await.unwrap();
            }
            Input::Seek(seek_time) => {
                PLAYER
                    .seek(Duration::from_millis(seek_time as u64), SeekMode::Absolute)
                    .await
                    .unwrap();
            }
            Input::Resume => {
                PLAYER.resume().await.unwrap();
            }
        }
    });
});
