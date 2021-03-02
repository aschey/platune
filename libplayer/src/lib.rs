mod actors;
mod channels;
mod context;
mod player_backend;
mod servo_backend;
mod util;
#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("features 'runtime-tokio' and 'runtime-async-std' are mutually exclusive");

pub mod libplayer {
    use crate::servo_backend::ServoBackend;
    use crate::{
        actors::{
            decoder::Decoder,
            player::Player,
            request_handler::{Command, RequestHandler},
            song_queue::SongQueue,
        },
        channels::mpsc::*,
    };
    use act_zero::{call, runtimes::default::spawn_actor};

    use gstreamer::glib::{self, MainLoop};
    use std::thread::{self, JoinHandle};

    pub struct PlatunePlayer {
        glib_main_loop: MainLoop,
        glib_handle: Option<JoinHandle<()>>,
        cmd_sender: Sender<Command>,
    }

    impl PlatunePlayer {
        pub async fn new() -> PlatunePlayer {
            let (tx, rx) = async_channel(32);
            let decoder_addr = spawn_actor(Decoder);
            let backend = Box::new(ServoBackend {});
            let player_addr = spawn_actor(Player::new(backend, decoder_addr));
            let player_addr_ = player_addr.clone();
            let queue_addr = spawn_actor(SongQueue::new(player_addr));
            let handler_addr = spawn_actor(RequestHandler::new(rx, queue_addr, player_addr_));

            let handler_task = async move {
                call!(handler_addr.run()).await.unwrap();
            };

            #[cfg(feature = "runtime-tokio")]
            tokio::spawn(handler_task);

            #[cfg(feature = "runtime-async-std")]
            async_std::task::spawn(handler_task);

            let main_loop = glib::MainLoop::new(None, false);
            let main_loop_ = main_loop.clone();
            let glib_handle = thread::spawn(move || {
                main_loop_.run();
            });
            PlatunePlayer {
                glib_main_loop: main_loop,
                glib_handle: Some(glib_handle),
                cmd_sender: tx,
            }
        }

        pub async fn set_queue(&self, queue: Vec<String>) {
            self.cmd_sender
                .send(Command::SetQueue(queue))
                .await
                .unwrap();
        }

        pub async fn seek(&self, seconds: f64) {
            self.cmd_sender.send(Command::Seek(seconds)).await.unwrap();
        }

        pub async fn set_volume(&self, volume: f32) {
            self.cmd_sender
                .send(Command::SetVolume(volume))
                .await
                .unwrap();
        }

        pub async fn pause(&self) {
            self.cmd_sender.send(Command::Pause).await.unwrap();
        }

        pub async fn resume(&self) {
            self.cmd_sender.send(Command::Resume).await.unwrap();
        }

        pub fn join(&mut self) {
            self.glib_main_loop.quit();
            self.glib_handle.take().unwrap().join().unwrap();
        }
    }
}
