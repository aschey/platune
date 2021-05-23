use crate::player_rpc::*;
use crate::player_server::Player;
use std::{pin::Pin, sync::Mutex};

use libplatune_player::libplayer::*;
use tonic::{Request, Response, Status};

pub struct PlayerImpl {
    player: Mutex<PlatunePlayer>,
    event_rx: broadcast::Receiver<PlayerEvent>,
}

impl PlayerImpl {
    pub fn new() -> PlayerImpl {
        let (platune, event_rx) = PlatunePlayer::new();

        PlayerImpl {
            player: Mutex::new(platune),
            event_rx,
        }
    }
}

#[tonic::async_trait]
impl Player for PlayerImpl {
    async fn set_queue(&self, request: Request<QueueRequest>) -> Result<Response<()>, Status> {
        let mut player = self.player.lock().unwrap();
        player.set_queue(request.into_inner().queue);
        Ok(Response::new(()))
    }

    async fn add_to_queue(
        &self,
        request: Request<AddToQueueRequest>,
    ) -> Result<Response<()>, Status> {
        let mut player = self.player.lock().unwrap();
        player.add_to_queue(request.into_inner().song);
        Ok(Response::new(()))
    }

    async fn pause(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.player.lock().unwrap().pause();
        Ok(Response::new(()))
    }

    async fn stop(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.player.lock().unwrap().stop();
        Ok(Response::new(()))
    }

    async fn resume(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.player.lock().unwrap().resume();

        Ok(Response::new(()))
    }

    async fn next(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.player.lock().unwrap().next();

        Ok(Response::new(()))
    }

    async fn previous(&self, _: Request<()>) -> Result<Response<()>, Status> {
        self.player.lock().unwrap().previous();

        Ok(Response::new(()))
    }

    async fn seek(&self, request: Request<SeekRequest>) -> Result<Response<()>, Status> {
        self.player
            .lock()
            .unwrap()
            .seek(request.into_inner().millis);

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
        let mut ended_rx = self.event_rx.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            while let Some(msg) = ended_rx.recv().await {
                match &msg {
                    PlayerEvent::SetVolume(volume) => tx
                        .send(Ok(EventResponse {
                            queue: vec![],
                            event: msg.to_string(),
                            millis: None,
                            volume: Some(*volume),
                        }))
                        .await
                        .unwrap_or_default(),
                    PlayerEvent::Seek(millis) => tx
                        .send(Ok(EventResponse {
                            queue: vec![],
                            event: msg.to_string(),
                            millis: Some(*millis),
                            volume: None,
                        }))
                        .await
                        .unwrap_or_default(),
                    PlayerEvent::StartQueue(queue) => tx
                        .send(Ok(EventResponse {
                            queue: queue.clone(),
                            event: msg.to_string(),
                            millis: None,
                            volume: None,
                        }))
                        .await
                        .unwrap_or_default(),
                    _ => tx
                        .send(Ok(EventResponse {
                            queue: vec![],
                            event: msg.to_string(),
                            millis: None,
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
