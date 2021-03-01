mod actors;
mod context;
mod player_backend;
mod servo_backend;

pub mod libplayer {

    use crate::actors::{decoder::Decoder, player::Player, song_queue::SongQueue};
    use crate::servo_backend::ServoBackend;
    use act_zero::{call, runtimes::default::spawn_actor};
    use gstreamer::glib;
    use std::thread;

    pub async fn run() {
        let handle = thread::spawn(|| {
            let main_loop = glib::MainLoop::new(None, false);
            main_loop.run();
        });
        let decoder_addr = spawn_actor(Decoder);
        let backend = ServoBackend {};
        let player_addr = spawn_actor(Player::new(backend, decoder_addr));
        let queue_addr = spawn_actor(SongQueue::new(player_addr));

        call!(queue_addr.set_queue(vec![
            "/home/aschey/windows/shared_files/Music/4 Strings/Believe/01 Intro.m4a".to_owned(),//"C:\\shared_files\\Music\\4 Strings\\Believe\\01 Intro.m4a".to_owned(),
            "/home/aschey/windows/shared_files/Music/4 Strings/Believe/02 Take Me Away (Into The Night).m4a"
                .to_owned()
        ]))
        .await
        .unwrap();
        handle.join().unwrap();
    }
}
