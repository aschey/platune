use std::time::Instant;

use act_zero::Actor;
use gstreamer::glib::subclass::simple::InstanceStruct;
use log::info;
use postage::{mpsc::Receiver, prelude::Stream};
use servo_media_audio::{analyser_node::AnalysisEngine, block::Block};

pub struct Analyser {
    data_rx: Receiver<Block>,
}

impl Analyser {
    pub fn new(data_rx: Receiver<Block>) -> Analyser {
        Analyser { data_rx }
    }
}

impl Actor for Analyser {}

impl Analyser {
    pub async fn run(&mut self) {
        let mut analysis_engine = AnalysisEngine::new(512, 0.6, -100.0, -10.0);
        let mut now = Instant::now();
        while let Some(data) = self.data_rx.recv().await {
            if now.elapsed().as_millis() >= 50 {
                analysis_engine.push(data);
                let mut data = [0u8; 512];
                analysis_engine.fill_byte_frequency_data(&mut data);
                now = Instant::now();
            }
        }
    }
}
