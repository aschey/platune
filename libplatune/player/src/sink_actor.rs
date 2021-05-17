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
use rodio::{Decoder, OutputStream, Sink as RodioSink};

use crate::event_loop::Command;

pub struct SinkActor {
    sink: RodioSink,
    queue: Vec<String>,
    position: usize,
    finish_tx: Sender<Receiver<()>>,
}

impl SinkActor {
    pub fn new(
        mut request_tx: Sender<Command>,
        finish_tx: Sender<Receiver<()>>,
        sink: RodioSink,
    ) -> Self {
        // let (_stream, handle) = OutputStream::try_default().unwrap();
        // let sink = RodioSink::try_new(&handle).unwrap();

        // tokio::task::spawn_blocking(move || async move {
        //     while let Some(receiver) = rx.recv().await {
        //         receiver.recv().unwrap();
        //         request_tx.send(Command::Ended).await.unwrap();
        //     }
        // });
        Self {
            sink,
            queue: vec![],
            position: 0,
            finish_tx,
        }
    }

    fn append_file(&self, path: String) {
        let file = File::open(path).unwrap();
        let decoder = Decoder::new(BufReader::new(file)).unwrap();
        self.sink.append(decoder);
    }

    pub fn start(&mut self) {
        // let (_stream, handle) = rodio::OutputStream::try_default().unwrap();
        // self.sink = rodio::Sink::try_new(&handle).unwrap();
        if let Some(path) = self.get_current() {
            self.append_file(path);
            self.signal_finish();
        }
        if let Some(path) = self.get_next() {
            self.append_file(path);
            self.signal_finish();
        }
        // self.sink.sleep_until_end();
    }

    pub fn play(&self) {
        self.sink.play();
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn seek(&self, time: Duration) {
        self.sink.seek(time);
    }

    pub fn stop(&self) {
        self.sink.stop();
    }

    pub fn on_ended(&mut self) {
        self.go_next();
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
            self.position += 1;
        }
    }

    pub fn go_previous(&mut self) {
        if self.position > 0 {
            self.position -= 1;
        }
    }
}
