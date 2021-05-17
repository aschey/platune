use std::{
    fs::File,
    io::BufReader,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

use act_zero::*;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink as RodioSink};

use crate::event_loop::Command;

pub struct SinkActor {
    sink: RodioSink,
    queue: Vec<String>,
    position: usize,
    finish_tx: Sender<Receiver<()>>,
    handle: OutputStreamHandle,
}

impl SinkActor {
    pub fn new(finish_tx: Sender<Receiver<()>>, handle: OutputStreamHandle) -> Self {
        let sink = rodio::Sink::try_new(&handle).unwrap();

        Self {
            sink,
            queue: vec![],
            position: 0,
            finish_tx,
            handle,
        }
    }

    fn append_file(&self, path: String) {
        let file = File::open(path).unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        self.sink.append(decoder);
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
    }

    pub fn play(&self) {
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn seek(&self, seconds: u64) {
        self.sink.seek(Duration::from_secs(seconds));
    }

    pub fn stop(&mut self) {
        self.reset();
        self.position = 0;
    }

    fn reset(&mut self) {
        self.sink.stop();
        self.sink = rodio::Sink::try_new(&self.handle).unwrap();
    }

    pub fn on_ended(&mut self) {
        if self.position < self.queue.len() - 1 {
            self.position += 1;
        }
        if let Some(file) = self.get_next() {
            self.append_file(file);
            self.signal_finish();
        }
    }

    fn signal_finish(&mut self) {
        let receiver = self.sink.get_current_receiver().unwrap();

        self.finish_tx.send(receiver).unwrap();
    }

    pub fn set_queue(&mut self, queue: Vec<String>) {
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
            self.reset();
            self.position += 1;
            self.start();
        }
    }

    pub fn go_previous(&mut self) {
        if self.position > 0 {
            self.reset();
            self.position -= 1;
            self.start();
        }
    }
}
