#![no_main]
use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};
use libplatune_player::platune_player::{PlatunePlayer, Settings};
use once_cell::sync::Lazy;
use std::{env::current_dir, time::Duration};
use tokio::runtime::Runtime;

#[derive(Arbitrary, Debug)]
enum Input {
    SetQueue,
    AddQueue,
    SetVolume(u8),
    Mute,
    Pause,
    Stop,
    Next,
    Previous,
    Seek(u16),
    Resume,
}

static PLAYER: Lazy<PlatunePlayer> = Lazy::new(|| {
    PlatunePlayer::new(Settings {
        enable_resampling: true,
        ..Default::default()
    })
});

static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

fn get_path(song: &str) -> String {
    let dir = current_dir().unwrap().to_str().unwrap().to_owned();
    format!("{dir}/../../test_assets/{song}")
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
            Input::SetQueue => {
                PLAYER.set_queue(vec![get_path("test.mp3")]).await.unwrap();
            }
            Input::AddQueue => {
                PLAYER
                    .add_to_queue(vec![get_path("test.mp3")])
                    .await
                    .unwrap();
            }
            Input::SetVolume(volume) => {
                PLAYER
                    .set_volume(((volume as f64) / 255.0).max(0.1))
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
                    .seek(Duration::from_millis(seek_time as u64))
                    .await
                    .unwrap();
            }
            Input::Resume => {
                PLAYER.resume().await.unwrap();
            }
        }
    });
});
