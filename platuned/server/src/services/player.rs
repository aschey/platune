use crate::player_server::Player;
use crate::rpc::*;
use std::{pin::Pin, sync::Arc};

use libplatune_player::platune_player::*;
use tonic::{Request, Response, Status};
use tracing::error;

pub struct PlayerImpl {
    player: Arc<PlatunePlayer>,
}

impl PlayerImpl {
    pub fn new(player: Arc<PlatunePlayer>) -> Self {
        PlayerImpl { player }
    }
}

fn format_error(msg: String) -> Status {
    error!("{:?}", msg);
    Status::internal(msg)
}

#[tonic::async_trait]
impl Player for PlayerImpl {
    async fn set_queue(&self, request: Request<QueueRequest>) -> Result<Response<()>, Status> {
        match self.player.set_queue(request.into_inner().queue) {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error setting queue: {:?}", e))),
        }
    }

    async fn add_to_queue(
        &self,
        request: Request<AddToQueueRequest>,
    ) -> Result<Response<()>, Status> {
        match self.player.add_to_queue(request.into_inner().songs) {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!(
                "Error adding songs to queue: {:?}",
                e
            ))),
        }
    }

    async fn pause(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.pause() {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error pausing queue: {:?}", e))),
        }
    }

    async fn stop(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.stop() {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error stopping queue: {:?}", e))),
        }
    }

    async fn resume(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.resume() {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error resuming queue: {:?}", e))),
        }
    }

    async fn next(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.next() {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!(
                "Error skipping to next song: {:?}",
                e
            ))),
        }
    }

    async fn previous(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.previous() {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!(
                "Error skipping to previous song: {:?}",
                e
            ))),
        }
    }

    async fn seek(&self, request: Request<SeekRequest>) -> Result<Response<()>, Status> {
        match self.player.seek(request.into_inner().millis) {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error seeking: {:?}", e))),
        }
    }

    async fn set_volume(&self, request: Request<SetVolumeRequest>) -> Result<Response<()>, Status> {
        match self.player.set_volume(request.into_inner().volume) {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error setting volume: {:?}", e))),
        }
    }

    type SubscribeEventsStream =
        Pin<Box<dyn futures::Stream<Item = Result<EventResponse, Status>> + Send + Sync + 'static>>;

    async fn subscribe_events(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
        let mut ended_rx = self.player.subscribe();
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            while let Ok(msg) = ended_rx.recv().await {
                match &msg {
                    PlayerEvent::SetVolume(volume) => tx
                        .send(Ok(EventResponse {
                            queue: vec![],
                            event: Event::SetVolume.into(),
                            millis: None,
                            volume: Some(*volume),
                        }))
                        .await
                        .unwrap_or_default(),
                    PlayerEvent::Seek(millis) => tx
                        .send(Ok(EventResponse {
                            queue: vec![],
                            event: Event::Seek.into(),
                            millis: Some(*millis),
                            volume: None,
                        }))
                        .await
                        .unwrap_or_default(),
                    PlayerEvent::StartQueue(queue) => tx
                        .send(Ok(EventResponse {
                            queue: queue.clone(),
                            event: Event::StartQueue.into(),
                            millis: None,
                            volume: None,
                        }))
                        .await
                        .unwrap_or_default(),
                    PlayerEvent::QueueUpdated(queue) => tx
                        .send(Ok(EventResponse {
                            queue: queue.clone(),
                            event: Event::QueueUpdated.into(),
                            millis: None,
                            volume: None,
                        }))
                        .await
                        .unwrap_or_default(),
                    _ => tx
                        .send(Ok(EventResponse {
                            queue: vec![],
                            event: match msg {
                                PlayerEvent::Stop => Event::Stop.into(),
                                PlayerEvent::Pause => Event::Pause.into(),
                                PlayerEvent::Resume => Event::Resume.into(),
                                PlayerEvent::Ended => Event::Ended.into(),
                                PlayerEvent::Next => Event::Next.into(),
                                PlayerEvent::Previous => Event::Previous.into(),
                                PlayerEvent::QueueEnded => Event::QueueEnded.into(),
                                _ => unreachable!(
                                    "Encountered unhandled event {:?}",
                                    msg.to_string()
                                ),
                            },
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
