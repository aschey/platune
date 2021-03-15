use std::{sync::Arc, time::Instant};

use act_zero::Actor;
use gstreamer::glib::subclass::simple::InstanceStruct;
use log::info;
use postage::{mpsc::Receiver, prelude::Stream};
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use servo_media_audio::{analyser_node::AnalysisEngine, block::Block};

pub struct Analyser {
    data_rx: Receiver<Block>,
    fft: Arc<dyn Fft<i16>>,
}

impl Analyser {
    pub fn new(data_rx: Receiver<Block>) -> Analyser {
        let mut planner = FftPlanner::<i16>::new();
        let fft = planner.plan_fft_forward(128);
        Analyser { data_rx, fft }
    }
}

impl Actor for Analyser {}

impl Analyser {
    pub async fn run(&mut self) {
        //let mut analysis_engine = AnalysisEngine::new(512, 0.6, -100.0, -10.0);
        let fft = self.fft.clone();
        let mut now = Instant::now();
        while let Some(mut data) = self.data_rx.recv().await {
            if now.elapsed().as_millis() >= 50 {
                let mut buffer = data
                    .as_mut_byte_slice()
                    .iter()
                    .map(|d| Complex {
                        re: d.to_owned() as i16,
                        im: d.to_owned() as i16,
                    })
                    .collect::<Vec<_>>();

                fft.process(&mut buffer);
                // info!("{:?}", buffer);
                // analysis_engine.push(data);
                // let mut data = [0u8; 512];
                // analysis_engine.fill_byte_frequency_data(&mut data);
                now = Instant::now();
            }
        }
    }
}
