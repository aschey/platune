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
use log::info;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink as RodioSink};

use crate::event_loop::Command;

pub struct SinkActor {
    sink: RodioSink,
    queue: Vec<String>,
    position: usize,
    finish_tx: Sender<Receiver<()>>,
    handle: OutputStreamHandle,
    ignore_count: usize,
    queued_count: usize,
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
            ignore_count: 0,
            queued_count: 0,
        }
    }

    fn append_file(&mut self, path: String) {
        let file = File::open(path).unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        self.sink.append(decoder);
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
        if self.position < self.queue.len() - 1 {
            self.position += 1;
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
        self.stop();
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
    }

    pub fn go_previous(&mut self) {
        if self.position > 0 {
            self.position -= 1;
            self.reset();
            self.start();
        }
    }
}
