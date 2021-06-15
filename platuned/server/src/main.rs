mod server;
use crate::player_server::PlayerServer;
use flexi_logger::{style, DeferredNow, LogTarget, Logger, Record};
use log::info;
use player_rpc::*;
use server::PlayerImpl;

use tonic::transport::Server;
use yansi::{Color, Style};

pub mod player_rpc {
    tonic::include_proto!("player_rpc");
    tonic::include_proto!("management_rpc");

    pub(crate) const FILE_DESCRIPTOR_SET: &'static [u8] =
        tonic::include_file_descriptor_set!("rpc_descriptor");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Logger::with_str("info")
        .format_for_stdout(colored)
        .log_target(LogTarget::StdOut)
        .set_palette("196;190;-;-;-".to_owned())
        .start()
        .unwrap();

    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(player_rpc::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let addr = "0.0.0.0:50051".parse().unwrap();

    let player = PlayerImpl::new();
    Server::builder()
        .add_service(service)
        .add_service(PlayerServer::new(player))
        .serve(addr)
        .await?;
    // /home/aschey/windows/shared_files/Music/emoisdead/Peu Etre - Langue Et Civilisation Hardcore (199x)/Peu Etre-17-Track 17.mp3
    // /home/aschey/windows/shared_files/Music/emoisdead/Peu Etre - Langue Et Civilisation Hardcore (199x)/Peu Etre-18-Track 18.mp3
    // C:\\shared_files\Music\emoisdead\Peu Etre - Langue Et Civilisation Hardcore (199x)\Peu Etre-17-Track 17.mp3
    // C:\\shared_files\Music\emoisdead\Peu Etre - Langue Et Civilisation Hardcore (199x)\Peu Etre-18-Track 18.mp3
    // client.set_queue(vec![
    //     // "/home/aschey/windows/shared_files/Music/4 Strings/Believe/01 Intro.m4a".to_owned(),
    //"C:\\shared_files\\Music\\4 Strings\\Believe\\01 Intro.m4a".to_owned(),
    //     // "/home/aschey/windows/shared_files/Music/4 Strings/Believe/02 Take Me Away (Into The Night).m4a"
    //     //     .to_owned()
    //     "/home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/04 Sun of Nothing.m4a".to_owned(),
    //     "/home/aschey/windows/shared_files/Music/Between the Buried and Me/Colors/05 Ants of the Sky.m4a".to_owned()
    // ]).await.unwrap();

    Ok(())
}

pub fn colored(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> core::result::Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "[{} {}] {} [{}:{}] {}",
        Style::new(Color::Cyan).paint(now.now().format("%Y-%m-%d %H:%M:%S%.6f")),
        Style::new(Color::RGB(119, 102, 204)).paint(now.now().timestamp_nanos() as f64 / 1e9),
        style(level, level),
        Style::new(Color::Green).paint(record.file().unwrap_or("<unnamed>")),
        Style::new(Color::Green).paint(record.line().unwrap_or(0)),
        style(level, &record.args())
    )
}
