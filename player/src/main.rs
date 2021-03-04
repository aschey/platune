use flexi_logger::{style, DeferredNow, LogTarget, Logger, Record};
use log::info;
use platune_libplayer::libplayer::PlatunePlayer;
use player_rpc::player_server::{Player, PlayerServer};
use player_rpc::{QueueRequest, SeekRequest, SetVolumeRequest};
use std::{
    borrow::BorrowMut,
    fs::read,
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tonic::{transport::Server, Request, Response, Status};
use yansi::{Color, Style};

pub mod player_rpc {
    tonic::include_proto!("player_rpc");

    pub(crate) const FILE_DESCRIPTOR_SET: &'static [u8] =
        tonic::include_file_descriptor_set!("player_rpc_descriptor");
}

pub struct PlayerImpl {
    player: PlatunePlayer,
}

#[tonic::async_trait]
impl Player for PlayerImpl {
    async fn set_queue(&self, request: Request<QueueRequest>) -> Result<Response<()>, Status> {
        self.player.set_queue(request.into_inner().queue).await;
        Ok(Response::new(()))
    }

    async fn pause(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.player.pause().await;
        Ok(Response::new(()))
    }

    async fn resume(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.player.resume().await;
        Ok(Response::new(()))
    }

    async fn seek(&self, request: Request<SeekRequest>) -> Result<Response<()>, Status> {
        self.player.seek(request.into_inner().time as f64).await;
        Ok(Response::new(()))
    }

    async fn set_volume(&self, request: Request<SetVolumeRequest>) -> Result<Response<()>, Status> {
        self.player.set_volume(request.into_inner().volume).await;
        Ok(Response::new(()))
    }
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
    let addr = "[::1]:50051".parse().unwrap();
    let platune = PlatunePlayer::new().await;
    let player = PlayerImpl { player: platune };
    Server::builder()
        .add_service(service)
        .add_service(PlayerServer::new(player))
        .serve(addr)
        .await?;

    // client.set_queue(vec![
    //     // "/home/aschey/windows/shared_files/Music/4 Strings/Believe/01 Intro.m4a".to_owned(),//"C:\\shared_files\\Music\\4 Strings\\Believe\\01 Intro.m4a".to_owned(),
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
        "[{}] {} [{}:{}] {}",
        Style::new(Color::Cyan).paint(now.now().format("%Y-%m-%d %H:%M:%S%.6f")),
        style(level, level),
        Style::new(Color::Green).paint(record.file().unwrap_or("<unnamed>")),
        Style::new(Color::Green).paint(record.line().unwrap_or(0)),
        style(level, &record.args())
    )
}
