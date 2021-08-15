use std::{
    fs::File,
    io::BufReader,
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

use log::info;
use rodio::{Decoder, OutputStreamHandle, Sink as RodioSink};
use tokio::sync::broadcast;

use crate::enums::PlayerEvent;
#[cfg(feature = "runtime-tokio")]
use crate::http_stream_reader::HttpStreamReader;

pub struct Player {
    sink: RodioSink,
    queue: Vec<String>,
    event_tx: broadcast::Sender<PlayerEvent>,
    position: usize,
    finish_tx: Sender<Receiver<()>>,
    handle: OutputStreamHandle,
    ignore_count: usize,
    queued_count: usize,
    volume: f32,
}

impl Player {
    pub fn new(
        finish_tx: Sender<Receiver<()>>,
        event_tx: broadcast::Sender<PlayerEvent>,
        handle: OutputStreamHandle,
    ) -> Self {
        let sink = rodio::Sink::try_new(&handle).unwrap();

        Self {
            sink,
            queue: vec![],
            event_tx,
            position: 0,
            finish_tx,
            handle,
            ignore_count: 0,
            queued_count: 0,
            volume: 0.5,
        }
    }

    fn append_file(&mut self, path: String) {
        if path.starts_with("http") {
            #[cfg(feature = "runtime-tokio")]
            {
                let reader = BufReader::new(HttpStreamReader::new(path));
                let decoder = Decoder::new(reader).unwrap();
                self.sink.append(decoder);
            }
        } else {
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);
            let decoder = Decoder::new(reader).unwrap();
            self.sink.append(decoder);
        }

        self.queued_count += 1;
        info!("Queued count {}", self.queued_count);
    }

    pub fn start(&mut self) {
        if let Some(path) = self.get_current() {
            self.append_file(path);
            self.signal_finish();
        }
        if let Some(path) = self.get_next() {
            self.append_file(path);
            self.signal_finish();
        }
        self.event_tx
            .send(PlayerEvent::StartQueue(self.queue.clone()))
            .unwrap();
    }

    pub fn play(&mut self) {
        self.sink.play();
        self.event_tx.send(PlayerEvent::Resume).unwrap();
    }

    pub fn pause(&mut self) {
        self.sink.pause();
        self.event_tx.send(PlayerEvent::Pause).unwrap();
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.sink.set_volume(volume);
        self.volume = volume;
    }

    pub fn seek(&mut self, millis: u64) {
        self.sink.seek(Duration::from_millis(millis));
        self.event_tx.send(PlayerEvent::Seek(millis)).unwrap();
    }

    pub fn stop(&mut self) {
        self.reset();
        self.position = 0;
        self.event_tx.send(PlayerEvent::Stop).unwrap();
    }

    fn ignore_ended(&mut self) {
        self.ignore_count = self.queued_count;

        info!("Ignore count {}", self.ignore_count);
    }

    fn reset(&mut self) {
        self.ignore_ended();
        self.sink.stop();
        self.sink = rodio::Sink::try_new(&self.handle).unwrap();
        self.sink.set_volume(self.volume);
    }

    pub fn on_ended(&mut self) {
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
        self.event_tx.send(PlayerEvent::Ended).unwrap();
        if self.position < self.queue.len() - 1 {
            self.position += 1;
            info!("Incrementing position. New position: {}", self.position);
        } else {
            self.event_tx.send(PlayerEvent::QueueEnded).unwrap();
        }

        if let Some(file) = self.get_next() {
            self.append_file(file);
            let receiver = self.sink.get_current_receiver().unwrap();
            self.finish_tx.send(receiver).unwrap();
        }
    }

    fn signal_finish(&mut self) {
        info!("Sending finish receiver");
        let receiver = self.sink.get_current_receiver().unwrap();
        self.finish_tx.send(receiver).unwrap();
    }

    pub fn set_queue(&mut self, queue: Vec<String>) {
        self.reset();
        self.position = 0;
        self.queue = queue;
        self.start();
    }

    pub fn add_to_queue(&mut self, song: String) {
        // Queue as not currently running, need to start it
        if self.queued_count == 0 {
            self.set_queue(vec![song]);
        } else {
            self.queue.push(song.clone());
            // Special case: if we started with only one song, then the new song will never get triggered by the ended event
            // so we need to add it here explicitly
            if self.queued_count == 1 {
                self.append_file(song);
                self.signal_finish();
            }

            self.event_tx
                .send(PlayerEvent::QueueUpdated(self.queue.clone()))
                .unwrap();
        }
    }

    pub fn get_current(&self) -> Option<String> {
        self.get_position(self.position)
    }

    pub fn get_next(&self) -> Option<String> {
        self.get_position(self.position + 1)
    }

    fn get_position(&self, position: usize) -> Option<String> {
        self.queue.get(position).map(String::to_owned)
    }

    pub fn go_next(&mut self) {
        if self.position < self.queue.len() - 1 {
            info!(
                "Current position: {}, Going to previous track.",
                self.position
            );
            self.position += 1;
            self.reset();
            self.start();
            self.event_tx.send(PlayerEvent::Next).unwrap();
        } else {
            info!("Already at beginning. Not going to previous track.");
        }
    }

    pub fn go_previous(&mut self) {
        if self.position > 0 {
            info!("Current position: {}, Going to next track.", self.position);
            self.position -= 1;
            self.reset();
            self.start();
            self.event_tx.send(PlayerEvent::Previous).unwrap();
        } else {
            info!(
                "Current position: {}. Already at end. Not going to next track.",
                self.position
            );
        }
    }
}
