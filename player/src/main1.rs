use gstreamer::prelude::*;
use gstreamer::*;
use gstreamer_app::AppSrc;
use io::Read;
use log::info;
use std::{fs::File, io, thread, time::Duration};
use std::{
    path::Path,
    sync::mpsc::{sync_channel, SyncSender},
};
use zerocopy::*;

#[allow(dead_code)]
pub struct GstreamerSink {
    tx: SyncSender<Vec<u8>>,
    pipeline: gstreamer::Pipeline,
}

impl GstreamerSink {
    fn open(device: Option<String>) -> GstreamerSink {
        gstreamer::init().expect("Failed to init gstreamer!");
        let pipeline_str_preamble = r#"appsrc caps="audio/x-raw,format=F32LE,layout=interleaved,channels=2,rate=44100" block=true max-bytes=4096 name=appsrc0 "#;
        let pipeline_str_rest = r#" ! audioconvert ! autoaudiosink"#;
        let pipeline_str: String = match device {
            Some(x) => format!("{}{}", pipeline_str_preamble, x),
            None => format!("{}{}", pipeline_str_preamble, pipeline_str_rest),
        };
        info!("Pipeline: {}", pipeline_str);

        gstreamer::init().unwrap();
        let pipelinee = gstreamer::parse_launch(&*pipeline_str).expect("Couldn't launch pipeline; likely a GStreamer issue or an error in the pipeline string you specified in the 'device' argument to librespot.");
        let pipeline = pipelinee
            .dynamic_cast::<gstreamer::Pipeline>()
            .expect("Couldn't cast pipeline element at runtime!");
        let bus = pipeline.get_bus().expect("Couldn't get bus from pipeline");
        let mainloop = glib::MainLoop::new(None, false);
        let appsrce: gstreamer::Element = pipeline
            .get_by_name("appsrc0")
            .expect("Couldn't get appsrc from pipeline");
        let appsrc: AppSrc = appsrce
            .dynamic_cast::<AppSrc>()
            .expect("Couldn't cast AppSrc element at runtime!");
        let bufferpool = gstreamer::BufferPool::new();
        let appsrc_caps = appsrc.get_caps().expect("Couldn't get appsrc caps");
        let mut conf = bufferpool.get_config();
        conf.set_params(Some(&appsrc_caps), 8192, 0, 0);
        bufferpool
            .set_config(conf)
            .expect("Couldn't configure the buffer pool");
        bufferpool
            .set_active(true)
            .expect("Couldn't activate buffer pool");

        let (tx, rx) = sync_channel::<Vec<u8>>(128);
        thread::spawn(move || {
            for data in rx {
                let buffer = bufferpool.acquire_buffer(None);
                if !buffer.is_err() {
                    let mut okbuffer = buffer.unwrap();
                    let mutbuf = okbuffer.make_mut();
                    mutbuf.set_size(data.len());
                    mutbuf
                        .copy_from_slice(0, data.as_bytes())
                        .expect("Failed to copy from slice");
                    let _eat = appsrc.push_buffer(okbuffer).unwrap();
                }
            }
        });

        thread::spawn(move || {
            let thread_mainloop = mainloop;
            let watch_mainloop = thread_mainloop.clone();
            bus.add_watch(move |_, msg| {
                match msg.view() {
                    MessageView::Eos(..) => watch_mainloop.quit(),
                    MessageView::Error(err) => {
                        println!(
                            "Error from {:?}: {} ({:?})",
                            err.get_src().map(|s| s.get_path_string()),
                            err.get_error(),
                            err.get_debug()
                        );
                        watch_mainloop.quit();
                    } //_ => (),
                    MessageView::Warning(_) => {}
                    MessageView::Info(_) => {}
                    MessageView::Tag(_) => {}
                    MessageView::Buffering(_) => {}
                    MessageView::StateChanged(_) => {}
                    MessageView::StateDirty(_) => {}
                    MessageView::StepDone(_) => {}
                    MessageView::ClockProvide(_) => {}
                    MessageView::ClockLost(_) => {}
                    MessageView::NewClock(_) => {}
                    MessageView::StructureChange(_) => {}
                    MessageView::StreamStatus(_) => {}
                    MessageView::Application(_) => {}
                    MessageView::Element(_) => {}
                    MessageView::SegmentStart(_) => {}
                    MessageView::SegmentDone(_) => {}
                    MessageView::DurationChanged(_) => {}
                    MessageView::Latency(_) => {}
                    MessageView::AsyncStart(_) => {}
                    MessageView::AsyncDone(_) => {}
                    MessageView::RequestState(_) => {}
                    MessageView::StepStart(_) => {}
                    MessageView::Qos(_) => {}
                    MessageView::Progress(_) => {}
                    MessageView::Toc(_) => {}
                    MessageView::ResetTime(_) => {}
                    MessageView::StreamStart(_) => {}
                    MessageView::NeedContext(_) => {}
                    MessageView::HaveContext(_) => {}
                    MessageView::DeviceAdded(_) => {}
                    MessageView::DeviceRemoved(_) => {}
                    MessageView::PropertyNotify(_) => {}
                    MessageView::StreamCollection(_) => {}
                    MessageView::StreamsSelected(_) => {}
                    MessageView::Redirect(_) => {}
                    MessageView::DeviceChanged(_) => {}
                    MessageView::Other => {}
                    MessageView::__NonExhaustive => {}
                };

                glib::Continue(true)
            })
            .expect("Failed to add bus watch");
            thread_mainloop.run();
        });

        pipeline
            .set_state(gstreamer::State::Playing)
            .expect("Unable to set the pipeline to the `Playing` state");

        GstreamerSink {
            tx: tx,
            pipeline: pipeline,
        }
    }

    fn start(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn stop(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn write(&mut self, data: &[u8]) -> io::Result<()> {
        // Copy expensively (in to_vec()) to avoid thread synchronization
        //let deighta: &[u8] = data.as_bytes();
        self.tx
            .send(data.to_vec())
            .expect("tx send failed in write function");
        Ok(())
    }
}

fn main() {
    let default = "C:\\shared_files\\Music\\4 Strings\\Believe\\01 Intro.m4a";
    let filename = default;
    let path = Path::new(filename);
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}", why),
        Ok(file) => file,
    };
    let mut sink = GstreamerSink::open(None);
    let mut buf: [u8; 4096] = [0; 4096];
    while let Ok(res) = file.read(&mut buf[..]) {
        if res == 0 {
            break;
        }
        sink.write(&buf).unwrap();
    }
    println!("here");

    thread::sleep(Duration::from_secs(1000));
}
