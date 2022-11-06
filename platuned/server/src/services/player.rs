use crate::player_server::Player;
use crate::rpc::event_response::*;
use crate::rpc::*;
use std::{pin::Pin, sync::Arc, time::Duration};

use daemon_slayer::{
    server::{BroadcastEventStore, EventStore, FutureExt, SubsystemHandle},
    signals::Signal,
};
use futures::StreamExt;
use libplatune_player::platune_player::*;
use tokio::sync::broadcast::error::RecvError;
use tonic::{Request, Response, Status};
use tracing::{error, info};

pub struct PlayerImpl {
    player: Arc<PlatunePlayer>,
    subsys: SubsystemHandle,
}

impl PlayerImpl {
    pub fn new(player: Arc<PlatunePlayer>, subsys: SubsystemHandle) -> Self {
        PlayerImpl { player, subsys }
    }
}

fn format_error(msg: String) -> Status {
    error!("{:?}", msg);
    Status::internal(msg)
}

fn get_event_response(event: Event, state: PlayerState) -> Result<EventResponse, Status> {
    Ok(EventResponse {
        event: event.into(),
        event_payload: Some(EventPayload::State(State {
            queue: state.queue,
            queue_position: state.queue_position as u32,
            volume: state.volume,
        })),
    })
}

fn map_response(msg: PlayerEvent) -> Result<EventResponse, Status> {
    match msg {
        PlayerEvent::Stop(state) => get_event_response(Event::Stop, state),
        PlayerEvent::Pause(state) => get_event_response(Event::Pause, state),
        PlayerEvent::Resume(state) => get_event_response(Event::Resume, state),
        PlayerEvent::Ended(state) => get_event_response(Event::Ended, state),
        PlayerEvent::Next(state) => get_event_response(Event::Next, state),
        PlayerEvent::StartQueue(state) => get_event_response(Event::StartQueue, state),
        PlayerEvent::QueueUpdated(state) => get_event_response(Event::QueueUpdated, state),
        PlayerEvent::Seek(state, time) => Ok(EventResponse {
            event: Event::Seek.into(),
            event_payload: Some(EventPayload::SeekData(SeekResponse {
                state: Some(State {
                    queue: state.queue,
                    queue_position: state.queue_position as u32,
                    volume: state.volume,
                }),
                seek_millis: time.as_millis() as u64,
            })),
        }),
        PlayerEvent::Previous(state) => get_event_response(Event::Previous, state),
        PlayerEvent::QueueEnded(state) => get_event_response(Event::QueueEnded, state),
        PlayerEvent::Position(position) => Ok(EventResponse {
            event: Event::Position.into(),
            event_payload: Some(EventPayload::Progress(PositionResponse {
                position: Some(
                    position
                        .position
                        .try_into()
                        .map_err(|e| format_error(format!("Error converting position: {e:?}")))?,
                ),
                retrieval_time: position
                    .retrieval_time
                    .map(|t| {
                        t.try_into().map_err(|e| {
                            format_error(format!("Error converting retrieval time: {e:?}"))
                        })
                    })
                    .map_or(Ok(None), |r| r.map(Some))?,
            })),
        }),
        _ => unreachable!("Encountered unhandled event {:?}", msg.to_string()),
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
        let status = self
            .player
            .get_current_status()
            .await
            .map_err(|e| format_error(format!("Error getting current status: {e:?}")))?;

        let progress =
            status
                .current_position
                .map(|p| {
                    Ok(PositionResponse {
                        position: Some(p.position.try_into().map_err(|e| {
                            format_error(format!("Error converting duration: {e:?}"))
                        })?),
                        retrieval_time: p
                            .retrieval_time
                            .map(|t| {
                                t.try_into().map_err(|e| {
                                    format_error(format!("Error converting retrieval time: {e:?}"))
                                })
                            })
                            .map_or(Ok(None), |r| r.map(Some))?,
                    })
                })
                .map_or(Ok(None), |r: Result<_, Status>| r.map(Some))?;

        Ok(Response::new(StatusResponse {
            progress,
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
        let subsys = self.subsys.clone();
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        tokio::spawn(async move {
            while let Ok(Ok(msg)) = ended_rx.recv().cancel_on_shutdown(&subsys).await {
                info!("Server received event {:?}", msg);
                let msg = map_response(msg);

                tx.send(msg).await.unwrap_or_default()
            }
        });
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }
}
