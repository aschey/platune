use flexi_logger::{style, DeferredNow, LogTarget, Logger, Record};
use log::info;
use platune_libplayer::libplayer::PlatunePlayer;
use yansi::{Color, Style};

#[tokio::main]
async fn main() {
    Logger::with_str("info")
        .format_for_stdout(colored)
        .log_target(LogTarget::StdOut)
        .set_palette("196;190;-;-;-".to_owned())
        .start()
        .unwrap();

    let mut player = PlatunePlayer::new().await;
    player.set_queue(vec![
            "/home/aschey/windows/shared_files/Music/4 Strings/Believe/01 Intro.m4a".to_owned(),//"C:\\shared_files\\Music\\4 Strings\\Believe\\01 Intro.m4a".to_owned(),
            "/home/aschey/windows/shared_files/Music/4 Strings/Believe/02 Take Me Away (Into The Night).m4a"
                .to_owned()
        ]).await;
    player.join();
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
