mod actors;
mod context;
mod player_backend;
mod servo_backend;
mod util;
#[cfg(all(feature = "runtime-tokio", feature = "runtime-async-std"))]
compile_error!("features 'runtime-tokio' and 'runtime-async-std' are mutually exclusive");

pub mod libplayer {
    use crate::actors::{
        analyser::Analyser,
        decoder::Decoder,
        player::Player,
        request_handler::{Command, RequestHandler},
        song_queue::SongQueue,
    };
    use crate::servo_backend::ServoBackend;
    use act_zero::{call, runtimes::default::spawn_actor};
    pub use postage::*;
    use strum_macros::Display;

    use gstreamer::glib::{self, MainLoop};
    pub use postage::{sink::Sink, stream::Stream};
    //use postage::{broadcast::Sender, mpsc, sink::Sink};
    use std::{
        fmt,
        thread::{self, JoinHandle},
    };

    pub struct PlatunePlayer {
        glib_main_loop: MainLoop,
        glib_handle: Option<JoinHandle<()>>,
        cmd_sender: mpsc::Sender<Command>,
    }

    impl PlatunePlayer {
        pub fn new(ended_tx: broadcast::Sender<PlayerEvent>) -> PlatunePlayer {
            let (tx, rx) = mpsc::channel(32);
            let (analysis_tx, analysis_rx) = mpsc::channel(32);
            let decoder_addr = spawn_actor(Decoder);
            let backend = Box::new(ServoBackend);
            let player_addr =
                spawn_actor(Player::new(backend, decoder_addr, ended_tx, analysis_tx));
            let player_addr_ = player_addr.clone();
            let queue_addr = spawn_actor(SongQueue::new(player_addr));
            let handler_addr = spawn_actor(RequestHandler::new(rx, queue_addr, player_addr_));
            let analyser_addr = spawn_actor(Analyser::new(analysis_rx));

            let handler_task = async move {
                call!(handler_addr.run()).await.unwrap();
            };

            let analyser_task = async move {
                call!(analyser_addr.run()).await.unwrap();
            };

            #[cfg(feature = "runtime-tokio")]
            {
                tokio::spawn(handler_task);
                tokio::spawn(analyser_task);
            }

            #[cfg(feature = "runtime-async-std")]
            {
                async_std::task::spawn(handler_task);
                async_std::task::spawn(analyser_task);
            }

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

        pub fn set_queue(&mut self, queue: Vec<String>) {
            self.cmd_sender.try_send(Command::SetQueue(queue)).unwrap();
        }

        pub fn seek(&mut self, seconds: f64) {
            self.cmd_sender.try_send(Command::Seek(seconds)).unwrap();
        }

        pub fn set_volume(&mut self, volume: f32) {
            self.cmd_sender
                .try_send(Command::SetVolume(volume))
                .unwrap();
        }

        pub fn pause(&mut self) {
            self.cmd_sender.try_send(Command::Pause).unwrap();
        }

        pub fn resume(&mut self) {
            self.cmd_sender.try_send(Command::Resume).unwrap();
        }

        pub fn join(&mut self) {
            self.glib_main_loop.quit();
            self.glib_handle.take().unwrap().join().unwrap();
        }
    }

    #[derive(Clone, Debug)]
    pub enum PlayerEvent {
        Pause { file: String },
        Play { file: String },
        Stop { file: String },
        Resume { file: String },
        Ended { file: String },
        SetVolume { file: String, volume: f32 },
        Seek { file: String, time: f64 },
    }
}
