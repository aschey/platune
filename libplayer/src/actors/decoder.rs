use crate::util::clocktime_to_seconds;
use gstreamer::subclass::error::FlowError::Error;

use std::{
    fs::File,
    io::Read,
    sync::{Arc, Mutex},
};

//use crate::context::CONTEXT;
use act_zero::*;
use futures::future::join;
use gstreamer::{
    glib::filename_to_uri, prelude::ObjectExt, ClockTime, ElementExt, ElementExtManual,
    ElementFactory, State,
};
use log::{error, info, warn};
use postage::{mpsc, prelude::Stream, sink::Sink};
use servo_media_audio::{
    context::{AudioContext, RealTimeAudioContextOptions},
    decoder::AudioDecoderCallbacks,
};

use super::gstreamer_context::GStreamerContext;

pub struct Decoder {
    context_addr: Addr<GStreamerContext>,
}

impl Actor for Decoder {}

impl Decoder {
    pub fn new(context_addr: Addr<GStreamerContext>) -> Decoder {
        Decoder { context_addr }
    }
    pub async fn decode(&self, filename: String) -> ActorResult<FileInfo> {
        let mut file = File::open(filename.to_owned()).unwrap();
        let mut bytes = vec![];

        file.read_to_end(&mut bytes).unwrap();
        let decoded_audio: Arc<Mutex<Vec<Vec<f32>>>> = Arc::new(Mutex::new(Vec::new()));
        let decoded_audio_ = decoded_audio.clone();
        let decoded_audio__ = decoded_audio.clone();
        let (mut sender, mut receiver) = mpsc::channel(32);
        let filename_ = filename.clone();
        let callbacks = AudioDecoderCallbacks::new()
            .eos(move || {
                info!("EOS");
                if let Err(err) = sender.try_send(()) {
                    warn!("Error sending EOS: {}", err);
                }
            })
            .error(|e| {
                error!("Error decoding audio {:?}", e);
            })
            .progress(move |buffer, mut channel| {
                if channel == 0 {
                    channel = 1;
                }
                let mut decoded_audio = decoded_audio_.lock().unwrap();
                decoded_audio[(channel - 1) as usize].extend_from_slice((*buffer).as_ref());
            })
            .ready(move |channels| {
                info!("Decoding {}, channels: {}", filename_, channels);
                decoded_audio__
                    .lock()
                    .unwrap()
                    .resize(channels as usize, Vec::new());
            })
            .build();
        call!(self.context_addr.decode_audio_data(bytes, callbacks))
            .await
            .unwrap();
        // context
        //     .lock()
        //     .unwrap()
        //     .decode_audio_data(bytes.to_vec(), callbacks);

        let (_, duration) = join(receiver.recv(), self.get_duration(&filename)).await;
        info!("Finished decoding");

        let RealTimeAudioContextOptions {
            sample_rate,
            latency_hint: _,
        } = RealTimeAudioContextOptions::default();
        let sample_rate = sample_rate as f64;
        let data = decoded_audio.lock().unwrap();
        let l = &data[0];
        let r = if data.len() > 1 { &data[1] } else { &data[0] };

        let start_gap = self.find_start_gap(l, r, sample_rate);
        let end_gap = self.find_end_gap(l, r, sample_rate);

        Produces::ok(FileInfo {
            data: data.to_vec(),
            start_gap,
            end_gap,
            duration,
            sample_rate,
        })
    }

    async fn get_duration(&self, filename: &str) -> f64 {
        info!("Reading duration");
        let fakesink = ElementFactory::make("fakesink", None).unwrap();
        let bin = ElementFactory::make("playbin", None).unwrap();

        bin.set_property("audio-sink", &fakesink).unwrap();
        let bus = bin.get_bus().unwrap();
        bus.add_signal_watch();
        let (sender, mut receiver) = mpsc::channel(32);
        let sender_mut = Mutex::new(sender);
        let bin_weak = bin.downgrade();
        let handler_id = bus
            .connect("message", false, move |_| {
                let bin = bin_weak.upgrade().unwrap();
                if let Some(duration) = bin.query_duration::<ClockTime>() {
                    if let Err(msg) = sender_mut.lock().unwrap().try_send(duration) {
                        warn!("{:?}", msg);
                    }
                }

                None
            })
            .unwrap();

        bin.set_property("uri", &filename_to_uri(filename, None).unwrap())
            .unwrap();

        bin.set_state(State::Playing).unwrap();
        let duration = receiver.recv().await.unwrap();
        let duration_seconds = clocktime_to_seconds(duration);
        info!("Got duration {:?}", duration_seconds);
        bus.disconnect(handler_id);
        bin.set_state(State::Null).unwrap();
        return duration_seconds;
    }

    fn find_start_gap(&self, l: &Vec<f32>, r: &Vec<f32>, sample_rate: f64) -> f64 {
        let duration = l.len();
        for i in 0..duration {
            if l[i] > 0. || r[i] > 0. {
                return i as f64 / sample_rate;
            }
        }

        return duration as f64;
    }

    fn find_end_gap(&self, l: &Vec<f32>, r: &Vec<f32>, sample_rate: f64) -> f64 {
        let duration = l.len();
        for i in (0..duration).rev() {
            if l[i] > 0. || r[i] > 0. {
                return (duration - i) as f64 / sample_rate;
            }
        }

        return duration as f64;
    }
}

#[derive(Debug)]
pub struct FileInfo {
    pub data: Vec<Vec<f32>>,
    pub start_gap: f64,
    pub end_gap: f64,
    pub duration: f64,
    pub sample_rate: f64,
}
