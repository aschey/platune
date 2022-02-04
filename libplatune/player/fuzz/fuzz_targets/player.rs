#![no_main]
use std::{env::current_dir, sync::Mutex, time::Duration};

use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};
use libplatune_player::platune_player::PlatunePlayer;
use once_cell::sync::Lazy;
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
}

static PLAYER: Lazy<Mutex<PlatunePlayer>> =
    Lazy::new(|| Mutex::new(PlatunePlayer::new(Default::default())));

static RUNTIME: Lazy<Mutex<Runtime>> = Lazy::new(|| Mutex::new(Runtime::new().unwrap()));

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
    RUNTIME.lock().unwrap().block_on(async {
        let player = PLAYER.lock().unwrap();

        match input {
            Input::SetQueue => {
                player.set_queue(vec![get_path("test.mp3")]).await.unwrap();
            }
            Input::AddQueue => {
                player
                    .add_to_queue(vec![get_path("test.mp3")])
                    .await
                    .unwrap();
            }
            Input::SetVolume(volume) => {
                player
                    .set_volume(((volume as f64) / 255.0).max(0.1))
                    .await
                    .unwrap();
            }
            Input::Mute => {
                player.set_volume(0.0).await.unwrap();
            }
            Input::Pause => {
                player.pause().await.unwrap();
            }
            Input::Stop => {
                player.stop().await.unwrap();
            }
            Input::Next => {
                player.next().await.unwrap();
            }
            Input::Previous => {
                player.previous().await.unwrap();
            }
        }

        tokio::time::sleep(Duration::from_millis(1000)).await;
    });
});
