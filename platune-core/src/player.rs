use std::{fs::File, io::BufReader, path::Path, time::Duration};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::{
    dto::{
        audio_status::AudioStatus, decoder_command::DecoderCommand,
        decoder_response::DecoderResponse, player_event::PlayerEvent, player_state::PlayerState,
        player_status::TrackStatus,
    },
    http_stream_reader::HttpStreamReader,
    settings::resample_mode::ResampleMode,
    source::{ReadSeekSource, Source},
    TwoWaySender,
};

pub(crate) struct Player {
    state: PlayerState,
    event_tx: broadcast::Sender<PlayerEvent>,
    queued_count: usize,
    queue_tx: crossbeam_channel::Sender<(Box<dyn Source>, ResampleMode)>,
    queue_rx: crossbeam_channel::Receiver<(Box<dyn Source>, ResampleMode)>,
    cmd_sender: TwoWaySender<DecoderCommand, DecoderResponse>,
    audio_status: AudioStatus,
    resample_mode: ResampleMode,
}

impl Player {
    pub(crate) fn new(
        event_tx: broadcast::Sender<PlayerEvent>,
        queue_tx: crossbeam_channel::Sender<(Box<dyn Source>, ResampleMode)>,
        queue_rx: crossbeam_channel::Receiver<(Box<dyn Source>, ResampleMode)>,
        cmd_sender: TwoWaySender<DecoderCommand, DecoderResponse>,
        resample_mode: ResampleMode,
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
            resample_mode,
        }
    }

    fn append_file(&mut self, path: String) {
        if path.starts_with("http") {
            let parts: Vec<&str> = path.rsplitn(2, '.').collect();
            let extension = if parts.len() > 1 {
                Some(parts[1].to_owned())
            } else {
                None
            };
            let http_reader = match HttpStreamReader::new(path.to_owned()) {
                Ok(http_reader) => http_reader,
                Err(e) => {
                    error!("Error downloading http file {:?}", e);
                    return;
                }
            };
            let file_len = http_reader.file_len;
            let reader = BufReader::new(http_reader);

            info!("Sending source {}", path);
            self.queue_tx
                .send((
                    Box::new(ReadSeekSource::new(reader, Some(file_len), extension)),
                    self.resample_mode.clone(),
                ))
                .unwrap();
        } else {
            let file = match File::open(&path) {
                Ok(file) => file,
                Err(e) => {
                    error!("Error opening file {:?}", e);
                    return;
                }
            };
            let len = file.metadata().unwrap().len();
            let extension = Path::new(&path)
                .extension()
                .map(|e| e.to_str().unwrap().to_owned());
            let reader = BufReader::new(file);

            info!("Sending source {}", path);
            self.queue_tx
                .send((
                    Box::new(ReadSeekSource::new(reader, Some(len), extension)),
                    self.resample_mode.clone(),
                ))
                .unwrap();
        }

        self.queued_count += 1;
        info!("Queued count {}", self.queued_count);
    }

    fn start(&mut self) {
        if let Some(path) = self.get_current() {
            self.append_file(path);
            self.audio_status = AudioStatus::Playing;

            self.event_tx
                .send(PlayerEvent::StartQueue(self.state.clone()))
                .unwrap_or_default();
        }
        if let Some(path) = self.get_next() {
            self.append_file(path);
        }
    }

    fn is_empty(&self) -> bool {
        self.state.queue.is_empty()
    }

    pub(crate) async fn play(&mut self) {
        if self.is_empty() {
            return;
        }
        self.cmd_sender.send(DecoderCommand::Play).await.unwrap();
        self.audio_status = AudioStatus::Playing;
        self.event_tx
            .send(PlayerEvent::Resume(self.state.clone()))
            .unwrap_or_default();
    }

    pub(crate) async fn pause(&mut self) {
        if self.is_empty() {
            return;
        }
        self.cmd_sender.send(DecoderCommand::Pause).await.unwrap();
        self.audio_status = AudioStatus::Paused;
        self.event_tx
            .send(PlayerEvent::Pause(self.state.clone()))
            .unwrap_or_default();
    }

    pub(crate) async fn set_volume(&mut self, volume: f64) {
        self.cmd_sender
            .send(DecoderCommand::SetVolume(volume))
            .await
            .unwrap();
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
            .unwrap()
        {
            DecoderResponse::SeekResponse(seek_result) => match seek_result {
                Some(seek_result) => {
                    info!("Seeked to {:?}", seek_result);
                    self.event_tx
                        .send(PlayerEvent::Seek(self.state.clone(), time))
                        .unwrap_or_default();
                }
                None => warn!("Error seeking"),
                //Err(e) => error!("Error receiving seek result: {:?}", e),
            },
            _ => unreachable!(),
        }
    }

    pub(crate) async fn stop(&mut self) {
        self.reset().await;
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

    async fn reset(&mut self) {
        // Get rid of any pending sources
        while self.queue_rx.try_recv().is_ok() {}
        self.audio_status = AudioStatus::Stopped;
        self.cmd_sender.send(DecoderCommand::Stop).await.unwrap();
    }

    pub(crate) fn on_ended(&mut self) {
        info!("Received ended event");
        self.queued_count -= 1;
        info!("Queued count {}", self.queued_count);

        if self.state.queue_position < self.state.queue.len() - 1 {
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
            self.append_file(file);
        }
    }

    pub(crate) async fn set_queue(&mut self, queue: Vec<String>) {
        // Don't need to send stop signal if no sources are playing
        if self.queued_count > 0 {
            self.reset().await;
        }

        self.state.queue_position = 0;
        self.state.queue = queue;
        self.start();
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
                self.append_file(song);
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
            self.reset().await;
            self.start();
            self.event_tx
                .send(PlayerEvent::Next(self.state.clone()))
                .unwrap_or_default();
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
            self.reset().await;
            self.start();
            self.event_tx
                .send(PlayerEvent::Previous(self.state.clone()))
                .unwrap_or_default();
        } else {
            info!(
                "Current position: {}. Already at beginning. Not going to previous track.",
                self.state.queue_position
            );
        }
    }
}
