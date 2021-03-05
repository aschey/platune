use flexi_logger::{style, DeferredNow, LogTarget, Logger, Record};
use log::info;
use platune_libplayer::libplayer::{broadcast, mpsc, sink::Sink, stream::Stream, PlatunePlayer};
use player_rpc::player_server::{Player, PlayerServer};
use player_rpc::{OnEndedResponse, QueueRequest, SeekRequest, SetVolumeRequest};
use std::{
    borrow::BorrowMut,
    fs::read,
    path::Path,
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
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
    player: Mutex<PlatunePlayer>,
    ended_tx: broadcast::Sender<String>,
}

#[tonic::async_trait]
impl Player for PlayerImpl {
    async fn set_queue(&self, request: Request<QueueRequest>) -> Result<Response<()>, Status> {
        self.player
            .lock()
            .unwrap()
            .set_queue(request.into_inner().queue);

        Ok(Response::new(()))
    }

    async fn pause(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.player.lock().unwrap().pause();
        Ok(Response::new(()))
    }

    async fn resume(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.player.lock().unwrap().resume();

        Ok(Response::new(()))
    }

    async fn seek(&self, request: Request<SeekRequest>) -> Result<Response<()>, Status> {
        self.player
            .lock()
            .unwrap()
            .seek(request.into_inner().time as f64);

        Ok(Response::new(()))
    }

    async fn set_volume(&self, request: Request<SetVolumeRequest>) -> Result<Response<()>, Status> {
        self.player
            .lock()
            .unwrap()
            .set_volume(request.into_inner().volume);
        Ok(Response::new(()))
    }

    type SubscribeOnEndedStream = Pin<
        Box<dyn futures::Stream<Item = Result<OnEndedResponse, Status>> + Send + Sync + 'static>,
    >;

    async fn subscribe_on_ended(
        &self,
        _: tonic::Request<()>,
    ) -> Result<Response<Self::SubscribeOnEndedStream>, Status> {
        let mut ended_rx = self.ended_tx.subscribe();
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            while let Some(msg) = ended_rx.recv().await {
                tx.send(Ok(OnEndedResponse { file: msg })).await.unwrap();
            }
        });
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
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
    let (tx, _) = broadcast::channel(32);
    let platune = PlatunePlayer::new(tx.clone());
    let player = PlayerImpl {
        player: Mutex::new(platune),
        ended_tx: tx,
    };
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
