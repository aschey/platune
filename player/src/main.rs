use flexi_logger::{style, DeferredNow, LogTarget, Logger, Record};
use jsonrpc_core::{
    futures::{self, lock::Mutex, FutureExt},
    BoxFuture, IoHandler, Value,
};
use jsonrpc_derive::rpc;
use log::info;
use platune_libplayer::libplayer::PlatunePlayer;
use std::{thread, time::Duration};
use yansi::{Color, Style};

#[tokio::main]
async fn main() {
    Logger::with_str("info")
        .format_for_stdout(colored)
        .log_target(LogTarget::StdOut)
        .set_palette("196;190;-;-;-".to_owned())
        .start()
        .unwrap();
    let player = PlatunePlayer::new().await;
    player.set_queue(vec![
                // "/home/aschey/windows/shared_files/Music/4 Strings/Believe/01 Intro.m4a".to_owned(),//"C:\\shared_files\\Music\\4 Strings\\Believe\\01 Intro.m4a".to_owned(),
                // "/home/aschey/windows/shared_files/Music/4 Strings/Believe/02 Take Me Away (Into The Night).m4a"
                //     .to_owned()
                "/home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/04 Sun of Nothing.m4a".to_owned(),
                "/home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a".to_owned()
            ]).await;
    // let p = Mutex::new(player);
    let mut io = IoHandler::<()>::default();
    // io.add_method("set_queue", |_params| async {
    //     player.set_queue(vec![
    //         // "/home/aschey/windows/shared_files/Music/4 Strings/Believe/01 Intro.m4a".to_owned(),//"C:\\shared_files\\Music\\4 Strings\\Believe\\01 Intro.m4a".to_owned(),
    //         // "/home/aschey/windows/shared_files/Music/4 Strings/Believe/02 Take Me Away (Into The Night).m4a"
    //         //     .to_owned()
    //         "/home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/04 Sun of Nothing.m4a".to_owned(),
    //         "/home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a".to_owned()
    //     ]).await;
    //     Ok(Value::String("hello".to_string()))
    // });
    // let _server = jsonrpc_ipc_server::ServerBuilder::new(io)
    //     .start("/tmp/parity-example.ipc")
    //     .expect("Server should start ok");

    //thread::sleep(Duration::from_secs(2));
    //player.pause().await;
    //thread::sleep(Duration::from_secs(2));
    //player.resume().await;
    //player.set_volume(1.).await;
    player.seek(60. * 10. + 55.).await;
    thread::sleep(Duration::from_secs(5000));
    //player.join();
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
