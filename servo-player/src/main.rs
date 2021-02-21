mod audio_node_wrapper;
mod media_element_node;
mod servo_backend;
use gstreamer::{
    glib::{self, filename_to_uri},
    prelude::ObjectExt,
    ClockTime, ElementExt, ElementFactory,
};
use gstreamer::{ElementExtManual, State};
use log::info;
use servo_media::{ClientContextId, ServoMedia};
use servo_media_audio::decoder::AudioDecoderCallbacks;
use servo_media_audio::node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage};
use servo_media_audio::{
    buffer_source_node::AudioBufferSourceNodeOptions,
    context::{AudioContext, AudioContextOptions, RealTimeAudioContextOptions},
};
use servo_media_audio::{
    buffer_source_node::{AudioBuffer, AudioBufferSourceNodeMessage},
    node::OnEndedCallback,
};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use tokio::sync::mpsc;

async fn run_example(context: &Arc<Mutex<AudioContext>>, start_time: f64, filename: &str) {
    let options = <RealTimeAudioContextOptions>::default();
    let sample_rate = options.sample_rate;
    let context = context.lock().unwrap();
    let args: Vec<_> = env::args().collect();
    //let default = "/home/aschey/windows/shared_files/Music/4 Strings/Believe/01 Intro.m4a";
    //let default = "C:\\shared_files\\Music\\4 Strings\\Believe\\01 Intro.m4a";
    // let filename: &str = if args.len() == 2 {
    //     args[1].as_ref()
    // } else if Path::new(default).exists() {
    //     default
    // } else {
    //     panic!("Usage: cargo run --bin audio_decoder <file_path>")
    // };
    let mut file = File::open(filename).unwrap();
    let mut bytes = vec![];

    file.read_to_end(&mut bytes).unwrap();
    let decoded_audio: Arc<Mutex<Vec<Vec<f32>>>> = Arc::new(Mutex::new(Vec::new()));
    let decoded_audio_ = decoded_audio.clone();
    let decoded_audio__ = decoded_audio.clone();
    let (sender, mut receiver) = mpsc::channel(32);
    let (sender2, mut receiver2) = mpsc::channel(32);
    let callbacks = AudioDecoderCallbacks::new()
        .eos(move || {
            sender2.try_send(()).unwrap();
        })
        .error(|e| {
            eprintln!("Error decoding audio {:?}", e);
        })
        .progress(move |buffer, channel| {
            let mut decoded_audio = decoded_audio_.lock().unwrap();
            decoded_audio[(channel - 1) as usize].extend_from_slice((*buffer).as_ref());
        })
        .ready(move |channels| {
            println!("There are {:?} audio channels", channels);
            decoded_audio__
                .lock()
                .unwrap()
                .resize(channels as usize, Vec::new());
            sender.try_send(()).unwrap();
        })
        .build();
    context.decode_audio_data(bytes.to_vec(), callbacks);
    println!("Decoding audio");
    receiver.recv().await.unwrap();
    println!("Audio decoded");
    let buffer_source = context.create_node(
        AudioNodeInit::AudioBufferSourceNode(Default::default()),
        Default::default(),
    );

    let dest = context.dest_node();
    context.connect_ports(buffer_source.output(0), dest.input(0));
    let callback = OnEndedCallback::new(|| {
        println!("Playback ended");
    });
    context.message_node(
        buffer_source,
        AudioNodeMessage::AudioScheduledSourceNode(
            AudioScheduledSourceNodeMessage::RegisterOnEndedCallback(callback),
        ),
    );
    context.message_node(
        buffer_source,
        AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(
            start_time,
        )),
    );
    // context.message_node(
    //     buffer_source,
    //     AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Stop()),
    // );
    receiver2.recv().await.unwrap();
    println!("{:?}", decoded_audio.lock().unwrap()[0][0]);
    context.message_node(
        buffer_source,
        AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetBuffer(Some(
            AudioBuffer::from_buffers(decoded_audio.lock().unwrap().to_vec(), sample_rate),
        ))),
    );
}

async fn get_duration(filename: &str) -> ClockTime {
    let fakesink = ElementFactory::make("fakesink", None).unwrap();
    let bin = ElementFactory::make("playbin", None).unwrap();
    bin.set_property("video-sink", &fakesink).unwrap();
    bin.set_property("audio-sink", &fakesink).unwrap();
    let bus = bin.get_bus().unwrap();
    bus.add_signal_watch();
    let (sender, mut receiver) = mpsc::channel(1);
    let bin_weak = bin.downgrade();
    let handler_id = bus
        .connect("message", false, move |_| {
            let bin = bin_weak.upgrade().unwrap();
            if let Some(duration) = bin.query_duration::<ClockTime>() {
                sender.try_send(duration).unwrap();
            }

            None
        })
        .unwrap();

    bin.set_property("uri", &filename_to_uri(filename, None).unwrap())
        .unwrap();
    bin.set_state(State::Playing).unwrap();
    let duration = receiver.recv().await.unwrap();
    bus.disconnect(handler_id);
    bin.set_state(State::Null).unwrap();
    return duration;
}

fn main() {
    let main_loop = glib::MainLoop::new(None, false);
    ServoMedia::init::<servo_media_auto::Backend>();
    if let Ok(servo_media) = ServoMedia::get() {
        let options = <RealTimeAudioContextOptions>::default();

        let context = servo_media.create_audio_context(
            &ClientContextId::build(1, 1),
            AudioContextOptions::RealTimeAudioContext(options),
        );

        run_example(
            &context,
            0.0,
            "C:\\shared_files\\Music\\4 Strings\\Believe\\01 Intro.m4a",
        );
        run_example(
            &context,
            54.729433106 - 0.04498866213151927,
            "C:\\shared_files\\Music\\4 Strings\\Believe\\02 Take Me Away (Into The Night).m4a",
        );
        let context = context.lock().unwrap();
        let _ = context.resume();
        main_loop.run();
        let _ = context.close();
    } else {
        unreachable!()
    }
}
