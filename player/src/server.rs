use crate::player_rpc::*;
use crate::player_server::Player;
use std::{pin::Pin, sync::Mutex};

use platune_libplayer::libplayer::*;
use tonic::{Request, Response, Status};

pub struct PlayerImpl {
    player: Mutex<PlatunePlayer>,
    ended_tx: broadcast::Sender<PlayerEvent>,
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

    type SubscribeEventsStream =
        Pin<Box<dyn futures::Stream<Item = Result<EventResponse, Status>> + Send + Sync + 'static>>;

    async fn subscribe_events(
        &self,
        _: tonic::Request<()>,
    ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
        let mut ended_rx = self.ended_tx.subscribe();
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            while let Some(msg) = ended_rx.recv().await {
                match &msg {
                    PlayerEvent::Play { file }
                    | PlayerEvent::Pause { file }
                    | PlayerEvent::Stop { file }
                    | PlayerEvent::Resume { file }
                    | PlayerEvent::Ended { file } => tx
                        .send(Ok(EventResponse {
                            file: file.to_owned(),
                            event: msg.to_string(),
                            time: None,
                            volume: None,
                        }))
                        .await
                        .unwrap_or_default(),

                    PlayerEvent::SetVolume { file, volume } => tx
                        .send(Ok(EventResponse {
                            file: file.to_owned(),
                            event: msg.to_string(),
                            time: None,
                            volume: Some(*volume),
                        }))
                        .await
                        .unwrap_or_default(),
                    PlayerEvent::Seek { file, time } => tx
                        .send(Ok(EventResponse {
                            file: file.to_owned(),
                            event: msg.to_string(),
                            time: Some(*time),
                            volume: None,
                        }))
                        .await
                        .unwrap_or_default(),
                }
            }
        });
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}
