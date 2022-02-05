use flume::{Receiver, Sender};
use std::{fs::File, io::BufReader, path::Path, time::Duration};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::{
    dto::{
        audio_status::AudioStatus, decoder_command::DecoderCommand,
        decoder_response::DecoderResponse, player_event::PlayerEvent, player_state::PlayerState,
        player_status::TrackStatus, queue_source::QueueSource,
    },
    http_stream_reader::HttpStreamReader,
    settings::Settings,
    source::{ReadSeekSource, Source},
    two_way_channel::TwoWaySender,
};

pub(crate) struct Player {
    state: PlayerState,
    event_tx: broadcast::Sender<PlayerEvent>,
    queued_count: usize,
    queue_tx: Sender<QueueSource>,
    queue_rx: Receiver<QueueSource>,
    cmd_sender: TwoWaySender<DecoderCommand, DecoderResponse>,
    audio_status: AudioStatus,
    settings: Settings,
    pending_volume: Option<f64>,
}

impl Player {
    pub(crate) fn new(
        event_tx: broadcast::Sender<PlayerEvent>,
        queue_tx: Sender<QueueSource>,
        queue_rx: Receiver<QueueSource>,
        cmd_sender: TwoWaySender<DecoderCommand, DecoderResponse>,
        settings: Settings,
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
        }
    }

    async fn get_source(&self, path: String) -> Option<Box<dyn Source>> {
        if path.starts_with("http") {
            info!("Creating http stream");
            let http_reader = match HttpStreamReader::new(path.to_owned()).await {
                Ok(http_reader) => http_reader,
                Err(e) => {
                    error!("Error downloading http file {e:?}");
                    return None;
                }
            };
            Some(http_reader.into_source())
        } else {
            let file = match File::open(&path) {
                Ok(file) => file,
                Err(e) => {
                    error!("Error opening file {e:?}");
                    return None;
                }
            };
            let file_len = match file.metadata() {
                Ok(metadata) => Some(metadata.len()),
                Err(e) => {
                    warn!("Error reading file metadata: {e:?}");
                    None
                }
            };
            let extension = Path::new(&path)
                    .extension()
                    .map(|e| match e.to_str() {
                        None => {
                            warn!("File extension for {path} contains invalid unicode. Not using extension hint");
                            None
                        },
                        extension => extension.map(|e| e.to_owned())
                    }).flatten();

            let reader = BufReader::new(file);

            Some(Box::new(ReadSeekSource::new(reader, file_len, extension)) as Box<dyn Source>)
        }
    }

    async fn append_file(&mut self, path: String, force_restart_output: bool) -> bool {
        match self.get_source(path.clone()).await {
            Some(source) => {
                info!("Sending source {path}");
                match self
                    .queue_tx
                    .send_async(QueueSource {
                        source,
                        settings: self.settings.clone(),
                        force_restart_output,
                        volume: self.pending_volume.take(),
                    })
                    .await
                {
                    Ok(()) => {
                        self.queued_count += 1;
                        info!("Queued count {}", self.queued_count);
                    }
                    Err(e) => {
                        error!("Error sending source {e:?}");
                    }
                }
                true
            }
            None => {
                let queue = self.state.queue.clone();
                self.state.queue = queue.into_iter().filter(|q| *q != path).collect();
                false
            }
        }
    }

    async fn start(&mut self, force_restart_output: bool) -> bool {
        let mut success = false;
        // Keep trying until a valid source is found or we reach the end of the queue
        while !success {
            match self.get_current() {
                Some(path) => {
                    success |= self.append_file(path.clone(), force_restart_output).await;

                    if let Some(path) = self.get_next() {
                        success |= self
                            .append_file(path, force_restart_output && !success)
                            .await;
                    }
                }
                None => return false,
            }
        }

        if success {
            self.wait_for_decoder().await;
            self.audio_status = AudioStatus::Playing;
        }

        success
    }

    async fn wait_for_decoder(&self) {
        if let Err(e) = self
            .cmd_sender
            .get_response(DecoderCommand::WaitForInitialization)
            .await
        {
            error!("Error receiving initialization response {e:?}");
        }
    }

    fn is_empty(&self) -> bool {
        self.state.queue.is_empty()
    }

    pub(crate) async fn play(&mut self) {
        if self.is_empty() {
            return;
        }
        if let Err(e) = self.cmd_sender.get_response(DecoderCommand::Play).await {
            error!("Error sending play command {e:?}");
        }
        self.audio_status = AudioStatus::Playing;
        self.event_tx
            .send(PlayerEvent::Resume(self.state.clone()))
            .unwrap_or_default();
    }

    pub(crate) async fn pause(&mut self) {
        if self.is_empty() {
            return;
        }
        if let Err(e) = self.cmd_sender.get_response(DecoderCommand::Pause).await {
            error!("Error sending pause command {e:?}");
        }
        self.audio_status = AudioStatus::Paused;
        self.event_tx
            .send(PlayerEvent::Pause(self.state.clone()))
            .unwrap_or_default();
    }

    pub(crate) async fn set_volume(&mut self, volume: f64) {
        if self.audio_status == AudioStatus::Stopped {
            // Decoder isn't running so we can't set the volume yet
            // This will get sent with the next source
            self.pending_volume = Some(volume);
        } else if let Err(e) = self
            .cmd_sender
            .get_response(DecoderCommand::SetVolume(volume))
            .await
        {
            error!("Error sending set volume command {e:?}");
        }
        self.state.volume = volume;
    }

    pub(crate) async fn seek(&mut self, time: Duration) {
        if self.is_empty() {
            return;
        }

        match self
            .cmd_sender
            .get_response(DecoderCommand::Seek(time))
            .await
        {
            Ok(DecoderResponse::SeekResponse(seek_result)) => match seek_result {
                Ok(seek_result) => {
                    info!("Seeked to {seek_result:?}");
                    self.event_tx
                        .send(PlayerEvent::Seek(self.state.clone(), time))
                        .unwrap_or_default();
                }
                Err(e) => warn!("Error seeking: {e:?}"),
            },
            Err(e) => error!("Error receiving seek result {e:?}"),
            _ => unreachable!("Should only receive SeekResponse"),
        }
    }

    pub(crate) async fn stop(&mut self) {
        self.reset_queue().await;
        self.state.queue_position = 0;
        self.state.queue = vec![];
        self.queued_count = 0;
        self.event_tx
            .send(PlayerEvent::Stop(self.state.clone()))
            .unwrap_or_default();
    }

    pub(crate) fn get_current_status(&self) -> TrackStatus {
        TrackStatus {
            status: self.audio_status.clone(),
            current_song: self.get_current(),
        }
    }

    async fn reset_queue(&mut self) {
        // Get rid of any pending sources
        self.queue_rx.drain();
        self.queued_count = 0;
        // If decoder is already stopped then sending additional stop events will cause the next song to skip
        if self.audio_status != AudioStatus::Stopped {
            info!("Sending decoder stop command");
            if let Err(e) = self.cmd_sender.get_response(DecoderCommand::Stop).await {
                error!("Error sending stop command {e:?}");
            } else {
                info!("Received stop response");
            }
        }
        self.audio_status = AudioStatus::Stopped;
    }

    pub(crate) async fn on_ended(&mut self) {
        info!("Received ended event");
        self.queued_count -= 1;
        info!("Queued count {}", self.queued_count);

        if self.state.queue_position < self.state.queue.len() - 1 {
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
            self.audio_status = AudioStatus::Stopped;
            self.event_tx
                .send(PlayerEvent::Ended(self.state.clone()))
                .unwrap_or_default();
            self.event_tx
                .send(PlayerEvent::QueueEnded(self.state.clone()))
                .unwrap_or_default();
        }

        if let Some(file) = self.get_next() {
            self.append_file(file, false).await;
        }
    }

    pub(crate) async fn reset(&mut self) {
        let queue = self.state.queue.clone();
        let queue_position = self.state.queue_position;
        self.set_queue_internal(queue, queue_position, true).await;
    }

    pub(crate) async fn set_queue(&mut self, queue: Vec<String>) {
        self.set_queue_internal(queue, 0, false).await;
        self.event_tx
            .send(PlayerEvent::StartQueue(self.state.clone()))
            .unwrap_or_default();
    }

    async fn set_queue_internal(
        &mut self,
        queue: Vec<String>,
        start_position: usize,
        force_restart_output: bool,
    ) {
        // Don't need to send stop signal if no sources are playing
        if self.queued_count > 0 {
            self.reset_queue().await;
        }

        self.state.queue_position = start_position;
        self.state.queue = queue;
        self.start(force_restart_output).await;
    }

    pub(crate) async fn add_to_queue(&mut self, songs: Vec<String>) {
        for song in songs {
            self.add_one_to_queue(song).await;
        }
    }

    async fn add_one_to_queue(&mut self, song: String) {
        // Queue is not currently running, need to start it
        if self.queued_count == 0 {
            self.set_queue(vec![song]).await;
        } else {
            self.state.queue.push(song.clone());
            // Special case: if we started with only one song, then the new song will never get triggered by the ended event
            // so we need to add it here explicitly
            if self.queued_count == 1 {
                self.append_file(song, false).await;
            }

            self.event_tx
                .send(PlayerEvent::QueueUpdated(self.state.clone()))
                .unwrap_or_default();
        }
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

    pub(crate) async fn go_next(&mut self) {
        let queue_len = self.state.queue.len();
        // need to check for length > 0 first because an unsigned value of 0 - 1 panics
        if queue_len > 0 && self.state.queue_position < queue_len - 1 {
            info!(
                "Current position: {}, Going to next track.",
                self.state.queue_position
            );
            self.state.queue_position += 1;
            self.reset_queue().await;
            if self.start(false).await {
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
    }

    pub(crate) async fn go_previous(&mut self) {
        if self.state.queue_position > 0 {
            info!(
                "Current position: {}, Going to previous track.",
                self.state.queue_position
            );
            self.state.queue_position -= 1;
            self.reset_queue().await;
            if self.start(false).await {
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
    }
}
