use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use decal::decoder::{ReadSeekSource, Source};
use flume::{Receiver, Sender};
use tap::{TapFallible, TapOptional};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::dto::audio_status::AudioStatus;
use crate::dto::decoder_command::DecoderCommand;
use crate::dto::decoder_response::DecoderResponse;
use crate::dto::player_event::PlayerEvent;
use crate::dto::player_state::PlayerState;
use crate::dto::player_status::TrackStatus;
use crate::dto::queue_source::{QueueSource, QueueStartMode};
use crate::http_stream_reader::HttpStreamReader;
use crate::settings::Settings;
use crate::two_way_channel::TwoWaySender;

#[derive(Debug)]
enum AppendError {
    SourceUnavailable,
    SendFailed,
}

pub(crate) struct Player {
    state: PlayerState,
    event_tx: broadcast::Sender<PlayerEvent>,
    queued_count: usize,
    queue_tx: Sender<QueueSource>,
    queue_rx: Receiver<QueueSource>,
    cmd_sender: TwoWaySender<DecoderCommand, DecoderResponse>,
    audio_status: AudioStatus,
    settings: Settings,
    pending_volume: Option<f32>,
    device_name: Option<String>,
}

impl Player {
    pub(crate) fn new(
        event_tx: broadcast::Sender<PlayerEvent>,
        queue_tx: Sender<QueueSource>,
        queue_rx: Receiver<QueueSource>,
        cmd_sender: TwoWaySender<DecoderCommand, DecoderResponse>,
        settings: Settings,
        device_name: Option<String>,
    ) -> Self {
        Self {
            event_tx,
            state: PlayerState {
                queue: vec![],
                volume: 0.5,
                queue_position: 0,
            },
            queued_count: 0,
            queue_tx,
            queue_rx,
            cmd_sender,
            audio_status: AudioStatus::Stopped,
            settings,
            pending_volume: None,
            device_name,
        }
    }

    async fn get_source(&self, path: String) -> Option<Box<dyn Source>> {
        if path.starts_with("http://") || path.starts_with("https://") {
            info!("Creating http stream");

            HttpStreamReader::new(path.to_owned())
                .await
                .map(|r| r.into_source())
                .tap_err(|e| error!("Error downloading http file {path} {e:?}"))
                .ok()
        } else {
            let file = File::open(&path)
                .tap_err(|e| error!("Error opening file {path} {e:?}"))
                .ok()?;

            let file_len = file
                .metadata()
                .map(|m| m.len())
                .tap_err(|e| warn!("Error reading file metadata from {path}: {e:?}"))
                .ok();

            let extension = Path::new(&path)
                .extension()
                .and_then(|ext| ext.to_str())
                .tap_none(|| {
                    warn!(
                        "File extension for {path} contains invalid unicode. Not using extension \
                         hint"
                    )
                })
                .map(|ext| ext.to_owned());

            let reader = BufReader::new(file);

            Some(Box::new(ReadSeekSource::new(reader, file_len, extension)) as Box<dyn Source>)
        }
    }

    async fn append_file(
        &mut self,
        path: String,
        queue_start_mode: QueueStartMode,
    ) -> Result<(), AppendError> {
        match self.get_source(path.clone()).await {
            Some(source) => {
                info!("Sending source {path}");
                match self
                    .queue_tx
                    .send_async(QueueSource {
                        source,
                        settings: self.settings.clone(),
                        queue_start_mode,
                        volume: self.pending_volume.take(),
                    })
                    .await
                {
                    Ok(()) => {
                        self.queued_count += 1;
                        info!("Queued count {}", self.queued_count);
                        Ok(())
                    }
                    Err(e) => {
                        error!("Error sending source {e:?}");
                        Err(AppendError::SendFailed)
                    }
                }
            }
            None => {
                let queue = self.state.queue.clone();
                self.state.queue = queue.into_iter().filter(|q| *q != path).collect();
                Err(AppendError::SourceUnavailable)
            }
        }
    }

    async fn start(&mut self, queue_start_mode: QueueStartMode) -> Result<(), Option<AppendError>> {
        let mut success: Result<(), Option<AppendError>> = Err(None);
        // Keep trying until a valid source is found or we reach the end of the queue
        while success.is_err() {
            match self.get_current() {
                Some(path) => {
                    match self
                        .append_file(path.clone(), queue_start_mode.clone())
                        .await
                    {
                        Ok(_) => {
                            success = Ok(());
                        }
                        Err(AppendError::SendFailed) => return Err(Some(AppendError::SendFailed)),
                        Err(AppendError::SourceUnavailable) => {}
                    }

                    if let Some(path) = self.get_next() {
                        let start_mode = match (&queue_start_mode, &success) {
                            (
                                QueueStartMode::ForceRestart {
                                    device_name,
                                    paused,
                                },
                                Err(_),
                            ) => QueueStartMode::ForceRestart {
                                device_name: device_name.to_owned(),
                                paused: *paused,
                            },
                            _ => QueueStartMode::Normal,
                        };
                        match self.append_file(path, start_mode).await {
                            Ok(_) => {
                                success = Ok(());
                            }
                            Err(AppendError::SendFailed) => {
                                return Err(Some(AppendError::SendFailed));
                            }
                            Err(AppendError::SourceUnavailable) => {}
                        }
                    }
                }
                None => return Err(None),
            }
        }

        info!("Waiting for decoder after starting");
        if success.is_ok()
            && self.wait_for_decoder().await == DecoderResponse::InitializationSucceeded
        {
            if matches!(queue_start_mode, QueueStartMode::ForceRestart {
                paused: true,
                ..
            }) {
                self.audio_status = AudioStatus::Paused;
            } else {
                self.audio_status = AudioStatus::Playing;
            }
        }

        success
    }

    async fn wait_for_decoder(&self) -> DecoderResponse {
        match self
            .cmd_sender
            .get_response(DecoderCommand::WaitForInitialization)
            .await
        {
            Ok(DecoderResponse::InitializationSucceeded) => {
                DecoderResponse::InitializationSucceeded
            }
            Ok(DecoderResponse::InitializationFailed) => {
                warn!("Decoder initialization failed");
                DecoderResponse::InitializationFailed
            }
            Ok(response) => {
                error!("Got unexpected decoder response: {:?}", response);
                response
            }
            Err(e) => {
                error!("Failed to get decoder response: {:?}", e);
                DecoderResponse::InitializationFailed
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.queued_count == 0
    }

    pub(crate) async fn play(&mut self) -> Result<(), String> {
        if self.is_empty() {
            info!("Play called on empty queue, ignoring");
            return Ok(());
        }

        self.cmd_sender
            .get_response(DecoderCommand::Play)
            .await
            .tap_err(|e| error!("Error sending play command {e:?}"))?;

        self.audio_status = AudioStatus::Playing;
        self.event_tx
            .send(PlayerEvent::Resume(self.state.clone()))
            .unwrap_or_default();

        Ok(())
    }

    pub(crate) async fn pause(&mut self) -> Result<(), String> {
        if self.is_empty() {
            info!("Pause called on empty queue, ignoring");
            return Ok(());
        }

        self.cmd_sender
            .get_response(DecoderCommand::Pause)
            .await
            .tap_err(|e| error!("Error sending pause command {e:?}"))?;

        self.audio_status = AudioStatus::Paused;
        self.event_tx
            .send(PlayerEvent::Pause(self.state.clone()))
            .unwrap_or_default();

        Ok(())
    }

    pub(crate) async fn set_volume(&mut self, volume: f32) -> Result<(), String> {
        if self.audio_status == AudioStatus::Stopped {
            // Decoder isn't running so we can't set the volume yet
            // This will get sent with the next source
            self.pending_volume = Some(volume);
        } else {
            self.cmd_sender
                .get_response(DecoderCommand::SetVolume(volume))
                .await
                .tap_err(|e| error!("Error sending set volume command {e:?}"))?;
        }

        self.state.volume = volume;
        Ok(())
    }

    pub(crate) async fn seek(&mut self, time: Duration) {
        if self.is_empty() {
            info!("Seek called on empty queue, ignoring");
            return;
        }

        match self
            .cmd_sender
            .get_response(DecoderCommand::Seek(time))
            .await
        {
            Ok(DecoderResponse::SeekResponse(Ok(seek_result))) => {
                info!("Seeked to {seek_result:?}");
                self.event_tx
                    .send(PlayerEvent::Seek(self.state.clone(), time))
                    .unwrap_or_default();
            }
            Ok(DecoderResponse::SeekResponse(Err(e))) => warn!("Error seeking: {e:?}"),
            Err(e) => error!("Error receiving seek result {e:?}"),
            _ => unreachable!("Should only receive SeekResponse"),
        }
    }

    pub(crate) async fn stop(&mut self) -> Result<(), String> {
        self.reset_queue().await?;
        self.state.queue_position = 0;
        self.state.queue = vec![];
        self.queued_count = 0;
        self.event_tx
            .send(PlayerEvent::Stop(self.state.clone()))
            .unwrap_or_default();

        Ok(())
    }

    pub(crate) fn get_current_status(&self) -> TrackStatus {
        TrackStatus {
            status: self.audio_status.clone(),
            current_song: self.get_current(),
        }
    }

    async fn reset_queue(&mut self) -> Result<(), String> {
        // Get rid of any pending sources
        self.queue_rx.drain();
        self.queued_count = 0;
        // If decoder is already stopped then sending additional stop events will cause the next
        // song to skip
        if self.audio_status != AudioStatus::Stopped {
            info!("Sending decoder stop command");
            self.cmd_sender
                .get_response(DecoderCommand::Stop)
                .await
                .tap_err(|e| error!("Error sending stop command {e:?}"))?;
            info!("Received stop response");
        }
        self.audio_status = AudioStatus::Stopped;
        Ok(())
    }

    pub(crate) async fn on_ended(&mut self) {
        info!("Received ended event");
        self.queued_count -= 1;
        info!("Queued count {}", self.queued_count);

        if self.state.queue_position < self.state.queue.len() - 1 {
            info!("Waiting for decoder after ended event");
            self.wait_for_decoder().await;
            self.state.queue_position += 1;
            self.event_tx
                .send(PlayerEvent::Ended(self.state.clone()))
                .unwrap_or_default();
            info!(
                "Incrementing position. New position: {}",
                self.state.queue_position
            );
        } else {
            info!("No more tracks in queue, changing to stopped state");
            self.audio_status = AudioStatus::Stopped;
            self.event_tx
                .send(PlayerEvent::Ended(self.state.clone()))
                .unwrap_or_default();
            self.event_tx
                .send(PlayerEvent::QueueEnded(self.state.clone()))
                .unwrap_or_default();
        }

        if let Some(file) = self.get_next() {
            self.append_file(file, QueueStartMode::Normal)
                .await
                .unwrap_or_default();
        }
    }

    pub(crate) async fn set_device_name(
        &mut self,
        device_name: Option<String>,
    ) -> Result<(), String> {
        self.device_name = device_name;
        self.reset().await
    }

    pub(crate) async fn reset(&mut self) -> Result<(), String> {
        let queue = self.state.queue.clone();
        let queue_position = self.state.queue_position;
        self.set_queue_internal(queue, queue_position, QueueStartMode::ForceRestart {
            device_name: self.device_name.clone(),
            paused: self.audio_status == AudioStatus::Paused,
        })
        .await?;

        Ok(())
    }

    pub(crate) async fn set_queue(&mut self, queue: Vec<String>) -> Result<(), String> {
        self.set_queue_internal(queue, 0, QueueStartMode::Normal)
            .await?;
        self.event_tx
            .send(PlayerEvent::StartQueue(self.state.clone()))
            .unwrap_or_default();

        Ok(())
    }

    async fn set_queue_internal(
        &mut self,
        queue: Vec<String>,
        start_position: usize,
        queue_start_mode: QueueStartMode,
    ) -> Result<(), String> {
        // Don't need to send stop signal if no sources are playing
        if self.queued_count > 0 {
            self.reset_queue().await?;
        }

        self.state.queue_position = start_position;
        self.state.queue = queue;

        match self.start(queue_start_mode).await {
            Err(Some(append_err)) => Err(format!("Failed to start queue: {append_err:?}")),
            _ => Ok(()),
        }
    }

    pub(crate) async fn add_to_queue(&mut self, songs: Vec<String>) -> Result<(), String> {
        for song in songs {
            self.add_one_to_queue(song).await?;
        }

        Ok(())
    }

    async fn add_one_to_queue(&mut self, song: String) -> Result<(), String> {
        // Queue is not currently running, need to start it
        if self.queued_count == 0 {
            self.set_queue(vec![song]).await?;
        } else {
            self.state.queue.push(song.clone());
            // Special case: if we started with only one song, then the new song will never get
            // triggered by the ended event so we need to add it here explicitly
            if self.queued_count == 1 {
                self.append_file(song, QueueStartMode::Normal)
                    .await
                    .unwrap_or_default();
            }

            self.event_tx
                .send(PlayerEvent::QueueUpdated(self.state.clone()))
                .unwrap_or_default();
        }

        Ok(())
    }

    fn get_current(&self) -> Option<String> {
        self.get_position(self.state.queue_position)
    }

    fn get_next(&self) -> Option<String> {
        self.get_position(self.state.queue_position + 1)
    }

    fn get_position(&self, position: usize) -> Option<String> {
        self.state.queue.get(position).map(String::to_owned)
    }

    pub(crate) async fn go_next(&mut self) -> Result<(), String> {
        let queue_len = self.state.queue.len();
        // need to check for length > 0 first because an unsigned value of 0 - 1 panics
        if queue_len > 0 && self.state.queue_position < queue_len - 1 {
            info!(
                "Current position: {}, Going to next track.",
                self.state.queue_position
            );
            self.state.queue_position += 1;
            self.reset_queue().await?;
            if self.start(QueueStartMode::Normal).await.is_ok() {
                self.event_tx
                    .send(PlayerEvent::Next(self.state.clone()))
                    .unwrap_or_default();
            }
        } else {
            info!(
                "Current position: {}. Already at end. Not going to next track.",
                self.state.queue_position
            );
        }

        Ok(())
    }

    pub(crate) async fn go_previous(&mut self) -> Result<(), String> {
        if self.state.queue_position > 0 {
            info!(
                "Current position: {}, Going to previous track.",
                self.state.queue_position
            );
            self.state.queue_position -= 1;
            self.reset_queue().await?;
            if self.start(QueueStartMode::Normal).await.is_ok() {
                self.event_tx
                    .send(PlayerEvent::Previous(self.state.clone()))
                    .unwrap_or_default();
            }
        } else {
            info!(
                "Current position: {}. Already at beginning. Not going to previous track.",
                self.state.queue_position
            );
        }

        Ok(())
    }
}
