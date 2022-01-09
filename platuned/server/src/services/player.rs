use crate::player_server::Player;
use crate::rpc::*;
use std::{pin::Pin, sync::Arc, time::Duration};

use platune_core::platune_player::*;
use tokio::sync::broadcast::{self, error::RecvError};
use tonic::{Request, Response, Status};
use tracing::{error, info};

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
        match self.player.set_queue(request.into_inner().queue).await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error setting queue: {:?}", e))),
        }
    }

    async fn add_to_queue(
        &self,
        request: Request<AddToQueueRequest>,
    ) -> Result<Response<()>, Status> {
        match self.player.add_to_queue(request.into_inner().songs).await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!(
                "Error adding songs to queue: {:?}",
                e
            ))),
        }
    }

    async fn pause(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.pause().await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error pausing queue: {:?}", e))),
        }
    }

    async fn stop(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.stop().await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error stopping queue: {:?}", e))),
        }
    }

    async fn resume(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.resume().await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error resuming queue: {:?}", e))),
        }
    }

    async fn next(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.next().await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!(
                "Error skipping to next song: {:?}",
                e
            ))),
        }
    }

    async fn previous(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.previous().await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!(
                "Error skipping to previous song: {:?}",
                e
            ))),
        }
    }

    async fn seek(&self, request: Request<SeekRequest>) -> Result<Response<()>, Status> {
        let time = request.into_inner().time.unwrap();
        let nanos = time.seconds * 1_000_000_000 + time.nanos as i64;
        match self.player.seek(Duration::from_nanos(nanos as u64)).await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error seeking: {:?}", e))),
        }
    }

    async fn set_volume(&self, request: Request<SetVolumeRequest>) -> Result<Response<()>, Status> {
        match self.player.set_volume(request.into_inner().volume).await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error setting volume: {:?}", e))),
        }
    }

    async fn get_current_status(&self, _: Request<()>) -> Result<Response<StatusResponse>, Status> {
        let status = self.player.get_current_status().await.unwrap();

        Ok(Response::new(StatusResponse {
            progress: status
                .current_time
                .current_time
                .map(|current_time| prost_types::Timestamp {
                    seconds: current_time.as_secs() as i64,
                    nanos: current_time.subsec_nanos() as i32,
                }),
            retrieval_time: status.current_time.retrieval_time.map(|retrieval_time| {
                prost_types::Timestamp {
                    seconds: retrieval_time.as_secs() as i64,
                    nanos: retrieval_time.subsec_nanos() as i32,
                }
            }),
            status: match status.track_status.status {
                AudioStatus::Playing => crate::rpc::PlayerStatus::Playing.into(),
                AudioStatus::Paused => crate::rpc::PlayerStatus::Paused.into(),
                AudioStatus::Stopped => crate::rpc::PlayerStatus::Stopped.into(),
            },
            current_song: status.track_status.current_song,
        }))
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
                info!("Server received event {:?}", msg);
                tx.send(Ok(match msg {
                    PlayerEvent::Stop(state) => get_event_response(Event::Stop, state, None),
                    PlayerEvent::Pause(state) => get_event_response(Event::Pause, state, None),
                    PlayerEvent::Resume(state) => get_event_response(Event::Resume, state, None),
                    PlayerEvent::Ended(state) => get_event_response(Event::Ended, state, None),
                    PlayerEvent::Next(state) => get_event_response(Event::Next, state, None),
                    PlayerEvent::StartQueue(state) => {
                        get_event_response(Event::StartQueue, state, None)
                    }
                    PlayerEvent::QueueUpdated(state) => {
                        get_event_response(Event::QueueUpdated, state, None)
                    }
                    PlayerEvent::Seek(state, time) => {
                        get_event_response(Event::Seek, state, Some(time.as_millis() as u64))
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
