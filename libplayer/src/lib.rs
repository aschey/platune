mod actors;
mod context;
mod player_backend;
mod servo_backend;

pub mod libplayer {

    use crate::actors::{
        decoder::Decoder,
        player::Player,
        request_handler::{Command, RequestHandler},
        song_queue::SongQueue,
    };
    use crate::servo_backend::ServoBackend;
    use act_zero::{call, runtimes::default::spawn_actor, Addr};
    use futures::{
        channel::mpsc::{self, Sender},
        SinkExt,
    };
    use gstreamer::glib;
    use std::thread::{self, JoinHandle};

    pub struct PlatunePlayer {
        glib_handle: Option<JoinHandle<()>>,
        cmd_sender: Sender<Command>,
    }

    impl PlatunePlayer {
        pub async fn new() -> PlatunePlayer {
            let (tx, rx) = mpsc::channel(32);
            let decoder_addr = spawn_actor(Decoder);
            let backend = Box::new(ServoBackend {});
            let player_addr = spawn_actor(Player::new(backend, decoder_addr));
            let queue_addr = spawn_actor(SongQueue::new(player_addr));
            let handler_addr = spawn_actor(RequestHandler::new(rx, queue_addr));

            let handler_task = async move {
                call!(handler_addr.run()).await.unwrap();
            };

            #[cfg(feature = "runtime-tokio")]
            tokio::spawn(handler_task);

            #[cfg(feature = "runtime-async-std")]
            async_std::task::spawn(handler_task);

            let glib_handle = thread::spawn(move || {
                let main_loop = glib::MainLoop::new(None, false);
                main_loop.run();
            });
            PlatunePlayer {
                glib_handle: Some(glib_handle),
                cmd_sender: tx,
            }
        }

        pub async fn set_queue(&mut self, queue: Vec<String>) {
            self.cmd_sender
                .send(Command::SetQueue(queue))
                .await
                .unwrap();
        }

        pub fn join(&mut self) {
            self.glib_handle.take().unwrap().join().unwrap();
        }
    }
}
