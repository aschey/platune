use crate::player_server::Player;
use crate::rpc::*;
use std::{pin::Pin, sync::Arc};

use libplatune_player::platune_player::*;
use tokio::sync::broadcast::{self, error::RecvError};
use tonic::{Request, Response, Status};
use tracing::error;

pub struct PlayerImpl {
    player: Arc<PlatunePlayer>,
    shutdown_tx: broadcast::Sender<()>,
}

impl PlayerImpl {
    pub fn new(player: Arc<PlatunePlayer>, shutdown_tx: broadcast::Sender<()>) -> Self {
        PlayerImpl {
            player,
            shutdown_tx,
        }
    }
}

fn format_error(msg: String) -> Status {
    error!("{:?}", msg);
    Status::internal(msg)
}

fn get_event_response(event: Event, state: PlayerState, seek_millis: Option<u64>) -> EventResponse {
    EventResponse {
        event: event.into(),
        queue: state.queue,
        queue_position: state.queue_position as u32,
        volume: state.volume,
        seek_millis,
    }
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
        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            while let Ok(msg) = tokio::select! { val = ended_rx.recv() => val, _ = shutdown_rx.recv() => Err(RecvError::Closed) }
            {
                tx.send(Ok(match msg {
                    PlayerEvent::Stop(state) => get_event_response(Event::Stop, state, None),
                    PlayerEvent::Pause(state) => get_event_response(Event::Pause, state, None),
                    PlayerEvent::Resume(state) => get_event_response(Event::Resume, state, None),
                    PlayerEvent::Ended(state) => get_event_response(Event::Ended, state, None),
                    PlayerEvent::Next(state) => get_event_response(Event::Next, state, None),
                    PlayerEvent::StartQueue(state) => {
                        get_event_response(Event::StartQueue, state, None)
                    }
                    PlayerEvent::Seek(state, seek_millis) => {
                        get_event_response(Event::Stop, state, Some(seek_millis))
                    }
                    PlayerEvent::Previous(state) => {
                        get_event_response(Event::Previous, state, None)
                    }
                    PlayerEvent::QueueEnded(state) => {
                        get_event_response(Event::QueueEnded, state, None)
                    }
                    _ => unreachable!("Encountered unhandled event {:?}", msg.to_string()),
                }))
                .await
                .unwrap_or_default()
            }
        });
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}
