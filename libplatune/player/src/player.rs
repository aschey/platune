use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use eyre::bail;
use flume::{Receiver, Sender};
use stream_download::registry::{Input, Registry};
use tap::TapFallible;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::dto::audio_status::AudioStatus;
use crate::dto::command::Command;
use crate::dto::decoder_command::DecoderCommand;
use crate::dto::decoder_response::DecoderResponse;
use crate::dto::player_event::PlayerEvent;
use crate::dto::player_response::PlayerResponse;
use crate::dto::player_state::PlayerState;
use crate::dto::player_status::TrackStatus;
use crate::dto::queue_source::{QueueSource, QueueStartMode};
use crate::dto::track::{Metadata, Track};
use crate::platune_player::SeekMode;
use crate::resolver::{
    DefaultUrlResolver, FileSourceResolver, HttpSourceResolver, MetadataSource, TrackInput,
    YtDlpSourceResolver, YtDlpUrlResolver,
};
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
    settings: Settings,
    pending_volume: Option<f32>,
    device_name: Option<String>,
    url_resolver: Registry<eyre::Result<Vec<Input>>>,
    source_resolver: Registry<eyre::Result<(MetadataSource, CancellationToken)>>,
    stream_cancellation_tokens: VecDeque<CancellationToken>,
}

impl Player {
    pub(crate) fn new(
        event_tx: broadcast::Sender<PlayerEvent>,
        queue_tx: Sender<QueueSource>,
        queue_rx: Receiver<QueueSource>,
        player_tx: TwoWaySender<Command, PlayerResponse>,
        cmd_sender: TwoWaySender<DecoderCommand, DecoderResponse>,
        settings: Settings,
        device_name: Option<String>,
    ) -> Self {
        Self {
            event_tx: event_tx.clone(),
            state: PlayerState {
                queue: vec![],
                volume: 1.0,
                queue_position: 0,
                status: AudioStatus::Stopped,
                metadata: None,
            },
            queued_count: 0,
            queue_tx,
            queue_rx,
            cmd_sender,
            settings,
            pending_volume: None,
            device_name,
            stream_cancellation_tokens: VecDeque::new(),
            url_resolver: Registry::new()
                .entry(YtDlpUrlResolver::new())
                .entry(DefaultUrlResolver::new()),
            source_resolver: Registry::new()
                .entry(HttpSourceResolver::new(Arc::new(move |metadata| {
                    let _ = player_tx
                        .send(Command::Metadata(metadata))
                        .inspect_err(|e| warn!("error sending metadata: {e:?}"));
                })))
                .entry(FileSourceResolver::new())
                .entry(YtDlpSourceResolver::new()),
        }
    }

    async fn get_source(&mut self, input: Input) -> Option<MetadataSource> {
        let (reader, cancellation_token) = self
            .source_resolver
            .find_match(input)
            .await?
            .tap_err(|e| error!("error resolving source: {e:?}"))
            .ok()?;
        self.stream_cancellation_tokens
            .push_back(cancellation_token);
        Some(reader)
    }

    async fn append_file(
        &mut self,
        input: TrackInput,
        queue_start_mode: QueueStartMode,
    ) -> Result<(), AppendError> {
        match self.get_source(input.input.clone()).await {
            Some(source) => {
                info!("Sending source {input:?}");
                match self
                    .queue_tx
                    .send_async(QueueSource {
                        source: source.source,
                        settings: self.settings.clone(),
                        queue_start_mode,
                        volume: self.pending_volume.take(),
                        // Metadata precedence:
                        // 1. Info supplied by the user
                        // 2. Extracted from the source
                        // 3. Default to the input URI as the song name
                        metadata: input
                            .metadata
                            .or(source.metadata)
                            .unwrap_or_else(|| Metadata {
                                song: input.input.to_string().into(),
                                ..Default::default()
                            }),
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
                error!("source unavailable");
                self.remove_from_queue(&input);
                Err(AppendError::SourceUnavailable)
            }
        }
    }

    fn remove_from_queue(&mut self, input: &TrackInput) {
        let queue = self.state.queue.clone();
        self.state.queue = queue.into_iter().filter(|q| q != input).collect();
    }

    async fn start(&mut self, queue_start_mode: QueueStartMode) -> Result<(), Option<AppendError>> {
        let mut success: Result<(), Option<AppendError>> = Err(None);
        // Keep trying until a valid source is found or we reach the end of the queue
        while success.is_err() {
            match self.get_current() {
                Some(input) => {
                    match self
                        .append_file(input.clone(), queue_start_mode.clone())
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

            info!("Waiting for decoder after starting");
            if success.is_ok() {
                let decoder_result = self.wait_for_decoder().await;
                if decoder_result == DecoderResponse::InitializationFailed {
                    warn!("received initialization failed message");
                    success = Err(Some(AppendError::SourceUnavailable));
                    let input = self.get_current().expect("current track missing");
                    self.stream_cancellation_tokens.pop_front();
                    self.queued_count -= 1;

                    self.remove_from_queue(&input);
                    continue;
                }

                if matches!(
                    queue_start_mode,
                    QueueStartMode::ForceRestart { paused: true, .. }
                ) {
                    self.state.status = AudioStatus::Paused;
                } else {
                    self.state.status = AudioStatus::Playing;
                }
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

        self.state.status = AudioStatus::Playing;
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

        self.state.status = AudioStatus::Paused;
        self.event_tx
            .send(PlayerEvent::Pause(self.state.clone()))
            .unwrap_or_default();

        Ok(())
    }

    pub(crate) async fn toggle(&mut self) -> Result<(), String> {
        if self.state.status == AudioStatus::Playing {
            self.pause().await
        } else {
            self.play().await
        }
    }

    pub(crate) async fn set_volume(&mut self, volume: f32) -> Result<(), String> {
        if self.state.status == AudioStatus::Stopped {
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

    pub(crate) fn update_metadata(&mut self, metadata: Metadata) {
        self.state.metadata = Some(metadata);
        let _ = self
            .event_tx
            .send(PlayerEvent::TrackChanged(self.state.clone()))
            .inspect_err(|e| warn!("error sending track changed {e:?}"));
    }

    pub(crate) async fn seek(&mut self, time: Duration, mode: SeekMode) {
        if self.is_empty() {
            info!("Seek called on empty queue, ignoring");
            return;
        }

        match self
            .cmd_sender
            .get_response(DecoderCommand::Seek(time, mode))
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
        self.state.metadata = None;
        self.queued_count = 0;
        self.event_tx
            .send(PlayerEvent::Stop(self.state.clone()))
            .unwrap_or_default();

        Ok(())
    }

    pub(crate) fn get_current_status(&self) -> TrackStatus {
        TrackStatus {
            status: self.state.status.clone(),
            state: self.state.clone(),
        }
    }

    async fn reset_queue(&mut self) -> Result<(), String> {
        // Get rid of any pending sources
        self.queue_rx.drain();
        for token in self.stream_cancellation_tokens.drain(..) {
            token.cancel();
        }
        self.queued_count = 0;
        // If decoder is already stopped then sending additional stop events will cause the next
        // song to skip
        if self.state.status != AudioStatus::Stopped {
            info!("Sending decoder stop command");
            self.cmd_sender
                .get_response(DecoderCommand::Stop)
                .await
                .tap_err(|e| error!("Error sending stop command {e:?}"))?;
            info!("Received stop response");
        }
        self.state.status = AudioStatus::Stopped;
        Ok(())
    }

    pub(crate) async fn on_ended(&mut self) {
        info!("Received ended event");
        self.queued_count -= 1;
        self.stream_cancellation_tokens.pop_front();
        info!("Queued count {}", self.queued_count);

        if self.state.queue_position < self.state.queue.len() - 1 {
            info!("Waiting for decoder after ended event");
            self.wait_for_decoder().await;
            self.state.queue_position += 1;
            info!(
                "Incrementing position. New position: {}",
                self.state.queue_position
            );
        } else {
            info!("No more tracks in queue, changing to stopped state");
            self.state.status = AudioStatus::Stopped;
            self.state.metadata = None;
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
        info!("resetting");
        self.set_queue_internal(
            queue,
            queue_position,
            QueueStartMode::ForceRestart {
                device_name: self.device_name.clone(),
                paused: self.state.status == AudioStatus::Paused,
            },
        )
        .await?;
        info!(
            "reset queue. length: {} position: {}",
            self.state.queue.len(),
            self.state.queue_position
        );
        Ok(())
    }

    async fn find_urls(&mut self, item: Track) -> eyre::Result<Vec<TrackInput>> {
        let Some(items) = self.url_resolver.find_match(&item.url).await else {
            bail!("no resolver found for {}", item.url);
        };
        let items = items?;
        Ok(items
            .into_iter()
            .map(|i| TrackInput {
                input: i,
                metadata: item.metadata.clone(),
            })
            .collect())
    }

    pub(crate) async fn set_queue(&mut self, queue: Vec<Track>) -> Result<(), String> {
        let mut new_queue = Vec::new();
        for item in queue {
            if let Ok(mut items) = self
                .find_urls(item)
                .await
                .tap_err(|e| error!("error finding urls: {e:?}"))
            {
                info!("url resolver found {} items", items.len());
                new_queue.append(&mut items);
            }
        }

        self.set_queue_internal(new_queue, 0, QueueStartMode::Normal)
            .await?;
        self.event_tx
            .send(PlayerEvent::StartQueue(self.state.clone()))
            .unwrap_or_default();

        Ok(())
    }

    async fn set_queue_internal(
        &mut self,
        queue: Vec<TrackInput>,
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

    pub(crate) async fn add_to_queue(&mut self, songs: Vec<Track>) -> Result<(), String> {
        for song in songs {
            if let Ok(urls) = self
                .find_urls(song)
                .await
                .tap_err(|e| error!("error finding urls {e:?}"))
            {
                for item in urls {
                    info!("url resolver found track: {item:?}");
                    self.add_one_to_queue(item).await?;
                }
            }
        }

        Ok(())
    }

    async fn add_one_to_queue(&mut self, song: TrackInput) -> Result<(), String> {
        // Queue is not currently running, need to start it
        if self.queued_count == 0 {
            self.set_queue_internal(vec![song], 0, QueueStartMode::Normal)
                .await?;
            self.event_tx
                .send(PlayerEvent::StartQueue(self.state.clone()))
                .unwrap_or_default();
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

    fn get_current(&self) -> Option<TrackInput> {
        self.get_position(self.state.queue_position)
    }

    fn get_next(&self) -> Option<TrackInput> {
        self.get_position(self.state.queue_position + 1)
    }

    fn get_position(&self, position: usize) -> Option<TrackInput> {
        self.state.queue.get(position).cloned()
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
                    .send(PlayerEvent::TrackChanged(self.state.clone()))
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
                    .send(PlayerEvent::TrackChanged(self.state.clone()))
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
