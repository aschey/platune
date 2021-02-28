mod actors;
mod context;
mod player_backend;
mod servo_backend;
use act_zero::{call, Addr};
use actors::{decoder::Decoder, player::Player, song_queue::SongQueue};
use flexi_logger::{style, DeferredNow, LogTarget, Logger, Record};
use futures::{executor::LocalPool, task::Spawn};
use gstreamer::glib;
use servo_backend::ServoBackend;
use std::thread;
use yansi::{Color, Style};

async fn run(spawner: &impl Spawn) {
    let decoder_addr = Addr::new(spawner, Decoder).unwrap();
    let backend = ServoBackend {};
    let player_addr = Addr::new(spawner, Player::new(backend, decoder_addr)).unwrap();
    let queue_addr = Addr::new(spawner, SongQueue::new(player_addr)).unwrap();

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

    let mut pool = LocalPool::new();
    let spawner = pool.spawner();

    pool.run_until(run(&spawner));
    handle.join().unwrap();
}
