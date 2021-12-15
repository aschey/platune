use std::{
    fs::File,
    io::{BufReader, Read, Seek},
    sync::mpsc::{Receiver, Sender},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use rodio::{Decoder, OutputStreamHandle, PlayError, Sink as RodioSink};
use tokio::sync::broadcast;
use tracing::{error, info};

#[cfg(feature = "runtime-tokio")]
use crate::http_stream_reader::HttpStreamReader;
use crate::{
    dto::{
        audio_status::AudioStatus, player_event::PlayerEvent, player_state::PlayerState,
        player_status::PlayerStatus,
    },
    timer::{Timer, TimerStatus},
};

pub(crate) struct Player {
    sink: RodioSink,
    state: PlayerState,
    event_tx: broadcast::Sender<PlayerEvent>,
    finish_tx: Sender<Receiver<()>>,
    handle: OutputStreamHandle,
    ignore_count: usize,
    queued_count: usize,
    current_time: Timer,
}

impl Player {
    pub(crate) fn new(
        finish_tx: Sender<Receiver<()>>,
        event_tx: broadcast::Sender<PlayerEvent>,
        handle: OutputStreamHandle,
    ) -> Result<Self, PlayError> {
        let sink = rodio::Sink::try_new(&handle)?;

        Ok(Self {
            sink,
            finish_tx,
            handle,
            event_tx,
            state: PlayerState {
                queue: vec![],
                volume: 0.5,
                queue_position: 0,
            },
            ignore_count: 0,
            queued_count: 0,
            current_time: Timer::default(),
        })
    }

    fn append_decoder<R: Read + Seek + Send + 'static>(&mut self, reader: R) {
        let decoder = match Decoder::new(reader) {
            Ok(decoder) => decoder,
            Err(e) => {
                error!("Error creating decoder {:?}", e);
                return;
            }
        };
        self.sink.append(decoder);
    }

    fn append_file(&mut self, path: String) {
        if path.starts_with("http") {
            #[cfg(feature = "runtime-tokio")]
            {
                let http_reader = match HttpStreamReader::new(path) {
                    Ok(http_reader) => http_reader,
                    Err(e) => {
                        error!("Error downloading http file {:?}", e);
                        return;
                    }
                };

                let reader = BufReader::new(http_reader);
                self.append_decoder(reader);
            }
        } else {
            let file = match File::open(path) {
                Ok(file) => file,
                Err(e) => {
                    error!("Error opening file {:?}", e);
                    return;
                }
            };
            let reader = BufReader::new(file);
            self.append_decoder(reader);
        }

        self.queued_count += 1;
        info!("Queued count {}", self.queued_count);
    }

    fn start(&mut self) {
        if let Some(path) = self.get_current() {
            self.append_file(path);
            self.signal_finish();
            self.current_time.start();

            self.event_tx
                .send(PlayerEvent::StartQueue(self.state.clone()))
                .unwrap_or_default();
        }
        if let Some(path) = self.get_next() {
            self.append_file(path);
            self.signal_finish();
        }
    }

    pub(crate) fn play(&mut self) {
        self.sink.play();
        self.current_time.resume();
        self.event_tx
            .send(PlayerEvent::Resume(self.state.clone()))
            .unwrap_or_default();
    }

    pub(crate) fn pause(&mut self) {
        self.sink.pause();
        self.current_time.pause();
        self.event_tx
            .send(PlayerEvent::Pause(self.state.clone()))
            .unwrap_or_default();
    }

    pub(crate) fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume);
        self.state.volume = volume;
    }

    pub(crate) fn seek(&mut self, millis: u64) {
        self.sink.seek(Duration::from_millis(millis));
        self.current_time.set_time(Duration::from_millis(millis));
        self.event_tx
            .send(PlayerEvent::Seek(self.state.clone(), millis))
            .unwrap_or_default();
    }

    pub(crate) fn stop(&mut self) {
        self.reset();
        self.state.queue_position = 0;
        self.current_time.stop();
        self.event_tx
            .send(PlayerEvent::Stop(self.state.clone()))
            .unwrap_or_default();
    }

    pub(crate) fn get_current_status(&self) -> PlayerStatus {
        let current_time = self.current_time.elapsed();
        PlayerStatus {
            current_time,
            retrieval_time: current_time
                .map(|_| SystemTime::now().duration_since(UNIX_EPOCH).ok())
                .flatten(),
            status: match self.current_time.status() {
                TimerStatus::Paused => AudioStatus::Paused,
                TimerStatus::Started => AudioStatus::Playing,
                TimerStatus::Stopped => AudioStatus::Stopped,
            },
            current_song: self.get_current(),
        }
    }

    fn ignore_ended(&mut self) {
        self.ignore_count = self.queued_count;

        info!("Ignore count {}", self.ignore_count);
    }

    fn reset(&mut self) {
        self.ignore_ended();
        self.sink.stop();
        self.current_time.stop();
        self.sink = match rodio::Sink::try_new(&self.handle) {
            Ok(sink) => sink,
            Err(e) => {
                error!("Error creating audio sink {:?}", e);
                return;
            }
        };
        self.sink.set_volume(self.state.volume);
    }

    pub(crate) fn on_ended(&mut self) {
        info!("Received ended event");
        self.queued_count -= 1;
        info!("Queued count {}", self.queued_count);
        if self.ignore_count > 0 {
            info!("Ignoring ended event");
            self.ignore_count -= 1;
            info!("Ignore count {}", self.ignore_count);
            return;
        } else {
            info!("Not ignoring ended event");
        }
        if self.state.queue_position < self.state.queue.len() - 1 {
            self.state.queue_position += 1;
            self.current_time.start();
            self.event_tx
                .send(PlayerEvent::Ended(self.state.clone()))
                .unwrap_or_default();
            info!(
                "Incrementing position. New position: {}",
                self.state.queue_position
            );
        } else {
            self.current_time.stop();
            self.event_tx
                .send(PlayerEvent::Ended(self.state.clone()))
                .unwrap_or_default();
            self.event_tx
                .send(PlayerEvent::QueueEnded(self.state.clone()))
                .unwrap_or_default();
        }

        if let Some(file) = self.get_next() {
            self.append_file(file);
            self.signal_finish();
        }
    }

    fn signal_finish(&mut self) {
        info!("Sending finish receiver");
        let receiver = match self.sink.get_current_receiver() {
            Some(receiver) => receiver,
            None => {
                error!("Unable to trigger song ended event because no receiver was found");
                return;
            }
        };
        if let Err(e) = self.finish_tx.send(receiver) {
            error!("Error sending song ended event {:?}", e);
        }
    }

    pub(crate) fn set_queue(&mut self, queue: Vec<String>) {
        self.reset();
        self.state.queue_position = 0;
        self.state.queue = queue;
        self.start();
    }

    pub(crate) fn add_to_queue(&mut self, songs: Vec<String>) {
        for song in songs {
            self.add_one_to_queue(song);
        }
    }

    fn add_one_to_queue(&mut self, song: String) {
        // Queue is not currently running, need to start it
        if self.queued_count == 0 {
            self.set_queue(vec![song]);
        } else {
            self.state.queue.push(song.clone());
            // Special case: if we started with only one song, then the new song will never get triggered by the ended event
            // so we need to add it here explicitly
            if self.queued_count == 1 {
                self.append_file(song);
                self.signal_finish();
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

    pub(crate) fn go_next(&mut self) {
        if self.state.queue_position < self.state.queue.len() - 1 {
            info!(
                "Current position: {}, Going to previous track.",
                self.state.queue_position
            );
            self.state.queue_position += 1;
            self.reset();
            self.start();
            self.event_tx
                .send(PlayerEvent::Next(self.state.clone()))
                .unwrap_or_default();
        } else {
            info!("Already at beginning. Not going to previous track.");
        }
    }

    pub(crate) fn go_previous(&mut self) {
        if self.state.queue_position > 0 {
            info!(
                "Current position: {}, Going to next track.",
                self.state.queue_position
            );
            self.state.queue_position -= 1;
            self.reset();
            self.start();
            self.event_tx
                .send(PlayerEvent::Previous(self.state.clone()))
                .unwrap_or_default();
        } else {
            info!(
                "Current position: {}. Already at end. Not going to next track.",
                self.state.queue_position
            );
        }
    }
}
