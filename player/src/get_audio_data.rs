use byte_slice_cast::AsSliceOf;
use gst::prelude::*;
use gst::GstBinExtManual;
use gstreamer as gst;
use gstreamer::{glib, prelude::Cast, Pipeline};
use gstreamer_app as gst_app;
use gstreamer_audio as gst_audio;
use gstreamer_player::{Player, PlayerGMainContextSignalDispatcher, PlayerSignalDispatcher};
use std::{
    fs::{self, File},
    io::{Cursor, Read, Seek, SeekFrom},
    sync::{mpsc, Arc, Mutex},
};

fn main() {
    let uri = "C:\\shared_files\\Music\\4 Strings\\Believe\\01 Intro.m4a";
    //let uri = "C:\\shared_files\\Music\\The Mars Volta\\Frances the Mute\\06 Cassandra Gemini.mp3";
    //let uri = "C:\\shared_files\\Music\\Between the Buried and Me\\Colors\\05 Ants of the Sky.m4a";
    let mut f = File::open(uri).unwrap();
    let metadata = fs::metadata(uri).unwrap();

    let mut data = vec![];
    //f.read_exact(&mut data).unwrap();
    f.read_to_end(&mut data).unwrap();
    f.seek(SeekFrom::Start(metadata.len() - 10000)).unwrap();
    let mut data2 = vec![];
    f.read_to_end(&mut data2).unwrap();
    //data.append(&mut data2);
    // let main_loop = glib::MainLoop::new(None, false);

    // let dispatcher = PlayerGMainContextSignalDispatcher::new(None);
    // let player = Player::new(None, Some(&dispatcher.upcast::<PlayerSignalDispatcher>()));

    // player.set_uri(uri);
    // player.play();
    // main_loop.run();
    gst::init().unwrap();
    let pipeline = Pipeline::new(None);
    //let callbacks = Arc::new(callbacks);

    let appsrc = gst::ElementFactory::make("appsrc", None).unwrap();

    let decodebin = gst::ElementFactory::make("decodebin", None).unwrap();

    // decodebin uses something called a "sometimes-pad", which is basically
    // a pad that will show up when a certain condition is met,
    // in decodebins case that is media being decoded
    pipeline.add_many(&[&appsrc, &decodebin]).unwrap();

    gst::Element::link_many(&[&appsrc, &decodebin]).unwrap();

    let appsrc = appsrc.downcast::<gst_app::AppSrc>().unwrap();

    //let options = options.unwrap_or_default();

    let (sender, receiver) = mpsc::channel();
    let sender = Arc::new(Mutex::new(sender));

    let pipeline_ = pipeline.downgrade();
    //let callbacks_ = callbacks.clone();
    //let sender_ = sender.clone();
    // Initial pipeline looks like
    //
    // appsrc ! decodebin2! ...
    //
    // We plug in the second part of the pipeline, including the deinterleave element,
    // once the media starts being decoded.
    decodebin.connect_pad_added(move |_, src_pad| {
        // A decodebin pad was added, if this is an audio file,
        // plug in a deinterleave element to separate each planar channel.
        //
        // Sub pipeline looks like
        //
        // ... decodebin2 ! audioconvert ! audioresample ! capsfilter ! deinterleave ...
        //
        // deinterleave also uses a sometime-pad, so we need to wait until
        // a pad for a planar channel is added to plug in the last part of
        // the pipeline, with the appsink that will be pulling the data from
        // each channel.

        //let callbacks = &callbacks_;
        //let sender = &sender_;
        let pipeline = pipeline_.upgrade().unwrap();

        // let caps = {
        //     let media_type = src_pad.get_current_caps().and_then(|caps| {
        //         caps.get_structure(0).map(|_| {
        //             //let name = s.get_name();
        //             caps.clone()
        //         })
        //     });

        //     match media_type {
        //         None => {
        //             // callbacks.error(AudioDecoderError::Backend(
        //             //     "Failed to get media type from pad".to_owned(),
        //             // ));
        //             // let _ = sender.lock().unwrap().send(());
        //             return;
        //         }
        //         Some(media_type) => media_type,
        //     }
        // };

        // if !is_audio {
        //     callbacks.error(AudioDecoderError::InvalidMediaFormat);
        //     let _ = sender.lock().unwrap().send(());
        //     return;
        // }

        //let sample_audio_info = gst_audio::AudioInfo::from_caps(&caps).unwrap();
        //let channels = sample_audio_info.channels();
        //callbacks.ready(channels);

        let insert_deinterleave = || -> () {
            let convert = gst::ElementFactory::make("audioconvert", None).unwrap();
            convert
                .set_property("mix-matrix", &gst::Array::new(&[]).to_value())
                .expect("mix-matrix property didn't work");
            let resample = gst::ElementFactory::make("audioresample", None).unwrap();
            //let filter = gst::ElementFactory::make("capsfilter", None).unwrap();
            let deinterleave =
                gst::ElementFactory::make("deinterleave", Some("deinterleave")).unwrap();

            deinterleave
                .set_property("keep-positions", &true.to_value())
                .expect("deinterleave doesn't have expected 'keep-positions' property");
            let pipeline_ = pipeline.downgrade();
            //let callbacks_ = callbacks.clone();
            deinterleave.connect_pad_added(move |_, src_pad| {
                // A new pad for a planar channel was added in deinterleave.
                // Plug in an appsink so we can pull the data from each channel.
                //
                // The end of the pipeline looks like:
                //
                // ... deinterleave ! queue ! appsink.
                //let callbacks = &callbacks_;
                let pipeline = pipeline_.upgrade().unwrap();
                let insert_sink = || -> () {
                    let queue = gst::ElementFactory::make("queue", None).unwrap();
                    let sink = gst::ElementFactory::make("appsink", None).unwrap();
                    let appsink = sink.clone().dynamic_cast::<gst_app::AppSink>().unwrap();
                    sink.set_property("sync", &false.to_value())
                        .expect("appsink doesn't handle expected 'sync' property");

                    //let callbacks_ = callbacks.clone();
                    appsink.set_callbacks(
                        gst_app::AppSinkCallbacks::builder()
                            .new_sample(move |appsink| {
                                let sample =
                                    appsink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                                //println!("{:?}", sample);
                                let buffer = sample.get_buffer_owned().unwrap();

                                let audio_info = sample
                                    .get_caps()
                                    .and_then(|caps| gst_audio::AudioInfo::from_caps(caps).ok())
                                    .unwrap();
                                let positions = audio_info.positions().unwrap();

                                for position in positions.iter() {
                                    let buffer = buffer.clone();
                                    let map = buffer.into_mapped_buffer_readable().unwrap();
                                    //let progress = Box::new(GStreamerAudioDecoderProgress(map));
                                    let channel = position.to_mask() as u32;
                                    let samples = map.as_slice_of::<i16>().unwrap();

                                    let sum: f64 = samples
                                        .iter()
                                        .map(|sample| {
                                            let f = f64::from(*sample) / f64::from(i16::MAX);
                                            f * f
                                        })
                                        .sum();

                                    println!("{:?} {:?} {:?}", sum, map, channel);
                                    //callbacks_.progress(progress, channel);
                                }

                                Ok(gst::FlowSuccess::Ok)
                            })
                            .build(),
                    );

                    let elements = &[&queue, &sink];
                    pipeline.add_many(elements).unwrap();
                    gst::Element::link_many(elements).unwrap();

                    for e in elements {
                        e.sync_state_with_parent().unwrap();
                    }

                    let sink_pad = queue.get_static_pad("sink").unwrap();
                    src_pad.link(&sink_pad).map(|_| ()).unwrap();
                };

                insert_sink();
            });

            // let mut audio_info_builder = gst_audio::AudioInfo::builder(
            //     gst_audio::AUDIO_FORMAT_F32,
            //     44_100,
            //     //options.sample_rate as u32,
            //     channels,
            // );
            // if let Some(positions) = sample_audio_info.positions() {
            //     audio_info_builder = audio_info_builder.positions(positions);
            // }
            //let audio_info = audio_info_builder.build().unwrap();
            //let caps = audio_info.to_caps().unwrap();
            // filter
            //     .set_property("caps", &caps)
            //     .expect("capsfilter doesn't have expected 'caps' property");

            let elements = &[&convert, &resample, &deinterleave];
            pipeline.add_many(elements).unwrap();
            gst::Element::link_many(elements).unwrap();

            for e in elements {
                e.sync_state_with_parent().unwrap();
            }

            let sink_pad = convert.get_static_pad("sink").unwrap();
            src_pad.link(&sink_pad).map(|_| ()).unwrap()
        };

        insert_deinterleave()
    });

    appsrc.set_property_format(gst::Format::Bytes);
    appsrc.set_property_block(true);

    let bus = pipeline.get_bus().unwrap();

    //let callbacks_ = callbacks.clone();
    // bus.set_sync_handler(move |_, msg| {
    //     use gst::MessageView;

    //     match msg.view() {
    //         MessageView::Error(e) => {
    //             // callbacks_.error(AudioDecoderError::Backend(
    //             //     e.get_debug().unwrap_or("Unknown".to_owned()),
    //             // ));
    //             println!("{:?}", e);
    //             let _ = sender.lock().unwrap().send(());
    //         }
    //         MessageView::Eos(_) => {
    //             println!("eos");
    //             //callbacks_.eos();
    //             let _ = sender.lock().unwrap().send(());
    //         }
    //         _ => (),
    //     }
    //     gst::BusSyncReply::Drop
    // });

    pipeline.set_state(gst::State::Playing).unwrap();

    let max_bytes = appsrc.get_max_bytes() as usize;
    let data_len = data.len();
    let mut reader = Cursor::new(data);
    while (reader.position() as usize) < data_len {
        let data_left = data_len - reader.position() as usize;
        let buffer_size = if data_left < max_bytes {
            data_left
        } else {
            max_bytes
        };
        let mut buffer = gst::Buffer::with_size(buffer_size).unwrap();
        {
            let buffer = buffer.get_mut().unwrap();
            let mut map = buffer.map_writable().unwrap();
            let mut buffer = map.as_mut_slice();
            let _ = reader.read(&mut buffer);
        }
        let _ = appsrc.push_buffer(buffer);
    }
    let _ = appsrc.end_of_stream();

    // Wait until we get an error or EOS.
    receiver.recv().unwrap();
    let _ = pipeline.set_state(gst::State::Null);
}
