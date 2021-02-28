mod actors;
mod context;
mod player_backend;
mod servo_backend;
use act_zero::{call, runtimes::default::spawn_actor};
use actors::{decoder::Decoder, player::Player, song_queue::SongQueue};
use flexi_logger::{style, DeferredNow, LogTarget, Logger, Record};
use gstreamer::glib;
use servo_backend::ServoBackend;
use std::thread;
use yansi::{Color, Style};

async fn run() {
    let decoder_addr = spawn_actor(Decoder);
    let backend = ServoBackend {};
    let player_addr = spawn_actor(Player::new(backend, decoder_addr));
    let queue_addr = spawn_actor(SongQueue::new(player_addr));

    call!(queue_addr.set_queue(vec![
        "C:\\shared_files\\Music\\4 Strings\\Believe\\01 Intro.m4a".to_owned(),
        "C:\\shared_files\\Music\\4 Strings\\Believe\\02 Take Me Away (Into the Night).m4a"
            .to_owned()
    ]))
    .await
    .unwrap();
}

pub fn colored(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "[{}] {} [{}:{}] {}",
        Style::new(Color::Cyan).paint(now.now().format("%Y-%m-%d %H:%M:%S%.6f")),
        style(level, level),
        Style::new(Color::Green).paint(record.file().unwrap_or("<unnamed>")),
        Style::new(Color::Green).paint(record.line().unwrap_or(0)),
        style(level, &record.args())
    )
}

#[tokio::main]
async fn main() {
    let handle = thread::spawn(|| {
        let main_loop = glib::MainLoop::new(None, false);
        main_loop.run();
    });

    Logger::with_str("info")
        .format_for_stdout(colored)
        .log_target(LogTarget::StdOut)
        .set_palette("196;190;-;-;-".to_owned())
        .start()
        .unwrap();

    run().await;
    handle.join().unwrap();
}
