use crate::{OnEndedResponse, Player, QueueRequest, SeekRequest, SetVolumeRequest};
use std::{pin::Pin, sync::Mutex};

use platune_libplayer::libplayer::{broadcast, PlatunePlayer, Stream};
use tonic::{Request, Response, Status};

pub struct PlayerImpl {
    player: Mutex<PlatunePlayer>,
    ended_tx: broadcast::Sender<String>,
}

impl PlayerImpl {
    pub fn new() -> PlayerImpl {
        let (tx, _) = broadcast::channel(32);
        let platune = PlatunePlayer::new(tx.clone());
        PlayerImpl {
            player: Mutex::new(platune),
            ended_tx: tx,
        }
    }
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
