use std::{
    fs::File,
    io::{BufReader, Read},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

use log::info;
use postage::sink::Sink;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink as RodioSink};

#[cfg(feature = "runtime-tokio")]
use crate::http_stream_reader::HttpStreamReader;
use crate::{event_loop::Command, libplayer::PlayerEvent};

pub struct Player {
    sink: RodioSink,
    queue: Vec<String>,
    event_tx: postage::broadcast::Sender<PlayerEvent>,
    position: usize,
    finish_tx: Sender<Receiver<()>>,
    handle: OutputStreamHandle,
    ignore_count: usize,
    queued_count: usize,
}

impl Player {
    pub fn new(
        finish_tx: Sender<Receiver<()>>,
        event_tx: postage::broadcast::Sender<PlayerEvent>,
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
            .try_send(PlayerEvent::StartQueue {
                queue: self.queue.clone(),
            })
            .unwrap();
    }

    pub fn play(&mut self) {
        self.sink.play();
        self.event_tx.try_send(PlayerEvent::Resume).unwrap();
    }

    pub fn pause(&mut self) {
        self.sink.pause();
        self.event_tx.try_send(PlayerEvent::Pause).unwrap();
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn seek(&mut self, millis: u64) {
        self.sink.seek(Duration::from_millis(millis));
        self.event_tx
            .try_send(PlayerEvent::Seek { millis })
            .unwrap();
    }

    pub fn stop(&mut self) {
        self.reset();
        self.position = 0;
        self.event_tx.try_send(PlayerEvent::Stop).unwrap();
    }

    fn ignore_ended(&mut self) {
        self.ignore_count = self.queued_count;
    }

    fn reset(&mut self) {
        self.ignore_ended();
        self.sink.stop();
        self.sink = rodio::Sink::try_new(&self.handle).unwrap();
    }

    pub fn on_ended(&mut self) {
        info!("ended");
        self.queued_count -= 1;
        if self.ignore_count > 0 {
            info!("ignoring");
            self.ignore_count -= 1;
            return;
        } else {
            info!("not ignoring");
        }
        self.event_tx.try_send(PlayerEvent::Ended).unwrap();
        if self.position < self.queue.len() - 1 {
            self.position += 1;
        } else {
            self.event_tx.try_send(PlayerEvent::QueueEnded).unwrap();
        }
        if let Some(file) = self.get_next() {
            self.append_file(file);
            let receiver = self.sink.get_current_receiver().unwrap();
            self.finish_tx.send(receiver).unwrap();
        }
    }

    fn signal_finish(&mut self) {
        let receiver = self.sink.get_current_receiver().unwrap();
        self.finish_tx.send(receiver).unwrap();
    }

    pub fn set_queue(&mut self, queue: Vec<String>) {
        self.reset();
        self.position = 0;
        self.queue = queue;
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
            self.position += 1;
            self.reset();
            self.start();
        }
        self.event_tx.try_send(PlayerEvent::Next).unwrap();
    }

    pub fn go_previous(&mut self) {
        if self.position > 0 {
            self.position -= 1;
            self.reset();
            self.start();
        }
        self.event_tx.try_send(PlayerEvent::Previous).unwrap();
    }
}
