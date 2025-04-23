use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use daemon_slayer::server::{BroadcastEventStore, EventStore, Signal};
use futures::StreamExt;
use libplatune_player::CpalOutput;
use libplatune_player::platune_player::*;
use tokio::sync::broadcast::error::RecvError;
use tonic::{Request, Response, Status};
use tracing::{error, info};

use crate::rpc::v1::event_response::*;
use crate::rpc::v1::{SeekMode, *};
use crate::v1::player_server::Player;

pub struct PlayerImpl {
    player: Arc<PlatunePlayer<CpalOutput>>,
    shutdown_rx: BroadcastEventStore<Signal>,
}

impl PlayerImpl {
    pub fn new(
        player: Arc<PlatunePlayer<CpalOutput>>,
        shutdown_rx: BroadcastEventStore<Signal>,
    ) -> Self {
        PlayerImpl {
            player,
            shutdown_rx,
        }
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
            queue: state.queue(),
            queue_position: state.queue_position as u32,
            volume: state.volume,
            status: map_audio_status(state.status).into(),
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
                    queue: state.queue(),
                    queue_position: state.queue_position as u32,
                    volume: state.volume,
                    status: map_audio_status(state.status).into(),
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

fn map_audio_status(status: AudioStatus) -> crate::rpc::v1::PlayerStatus {
    match status {
        AudioStatus::Playing => crate::rpc::v1::PlayerStatus::Playing,
        AudioStatus::Paused => crate::rpc::v1::PlayerStatus::Paused,
        AudioStatus::Stopped => crate::rpc::v1::PlayerStatus::Stopped,
    }
}

#[tonic::async_trait]
impl Player for PlayerImpl {
    async fn set_queue(&self, request: Request<QueueRequest>) -> Result<Response<()>, Status> {
        match self.player.set_queue(request.into_inner().queue).await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error setting queue: {e:?}"))),
        }
    }

    async fn add_to_queue(
        &self,
        request: Request<AddToQueueRequest>,
    ) -> Result<Response<()>, Status> {
        match self.player.add_to_queue(request.into_inner().songs).await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error adding songs to queue: {e:?}"))),
        }
    }

    async fn pause(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.pause().await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error pausing queue: {e:?}"))),
        }
    }

    async fn toggle(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.toggle().await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error pausing queue: {e:?}"))),
        }
    }

    async fn stop(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.stop().await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error stopping queue: {e:?}"))),
        }
    }

    async fn resume(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.resume().await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error resuming queue: {e:?}"))),
        }
    }

    async fn next(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.next().await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error skipping to next song: {e:?}"))),
        }
    }

    async fn previous(&self, _: Request<()>) -> Result<Response<()>, Status> {
        match self.player.previous().await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!(
                "Error skipping to previous song: {e:?}"
            ))),
        }
    }

    async fn seek(&self, request: Request<SeekRequest>) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        let time = request.time.unwrap();
        let mode = match request.mode() {
            SeekMode::Absolute => libplatune_player::platune_player::SeekMode::Absolute,
            SeekMode::Forward => libplatune_player::platune_player::SeekMode::Forward,
            SeekMode::Backward => libplatune_player::platune_player::SeekMode::Backward,
        };
        let nanos = time.seconds * 1_000_000_000 + time.nanos as i64;
        match self
            .player
            .seek(Duration::from_nanos(nanos as u64), mode)
            .await
        {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error seeking: {e:?}"))),
        }
    }

    async fn set_volume(&self, request: Request<SetVolumeRequest>) -> Result<Response<()>, Status> {
        match self.player.set_volume(request.into_inner().volume).await {
            Ok(()) => Ok(Response::new(())),
            Err(e) => Err(format_error(format!("Error setting volume: {e:?}"))),
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
            state: Some(State {
                queue: status.track_status.state.queue(),
                queue_position: status.track_status.state.queue_position as u32,
                volume: status.track_status.state.volume,
                status: map_audio_status(status.track_status.status).into(),
            }),
        }))
    }

    async fn list_output_devices(
        &self,
        _: Request<()>,
    ) -> Result<Response<DevicesResponse>, Status> {
        let devices = self
            .player
            .output_devices()
            .map_err(|e| format_error(format!("Error getting output devices: {e:?}")))?;
        Ok(Response::new(DevicesResponse { devices }))
    }

    async fn set_output_device(
        &self,
        request: Request<SetOutputDeviceRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();
        self.player
            .set_output_device(request.device)
            .await
            .map_err(|e| format_error(format!("Error setting output device: {e:?}")))?;
        Ok(Response::new(()))
    }

    type SubscribeEventsStream =
        Pin<Box<dyn futures::Stream<Item = Result<EventResponse, Status>> + Send + Sync + 'static>>;

    async fn subscribe_events(
        &self,
        _: Request<()>,
    ) -> Result<Response<Self::SubscribeEventsStream>, Status> {
        let mut player_rx = self.player.subscribe();
        let mut shutdown_rx = self.shutdown_rx.subscribe_events();
        let (tx, rx) = tokio::sync::mpsc::channel(32);
        let status = self
            .player
            .get_current_status()
            .await
            .map_err(|e| format_error(format!("error getting status: {e:?}")))?;

        let initial_message = match status.track_status.status {
            AudioStatus::Playing => map_response(PlayerEvent::Resume(status.track_status.state)),
            AudioStatus::Paused => map_response(PlayerEvent::Pause(status.track_status.state)),
            AudioStatus::Stopped => map_response(PlayerEvent::Stop(status.track_status.state)),
        };
        tx.send(initial_message).await.unwrap_or_default();
        if let Some(position) = status.current_position {
            let progress_message = map_response(PlayerEvent::Position(position));
            tx.send(progress_message).await.unwrap_or_default();
        }
        tokio::spawn(async move {
            while let Ok(msg) = tokio::select! {
                val = player_rx.recv() => val,
                _ = shutdown_rx.next() => Err(RecvError::Closed)
            } {
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
