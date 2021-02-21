use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
};

use ipc::IpcReceiver;
use servo_media::{
    player::{
        context::{GlApi, GlContext, NativeDisplay, PlayerGLContext},
        ipc_channel::ipc,
        Player, PlayerEvent, StreamType,
    },
    ClientContextId, ServoMedia,
};
use servo_media_audio::{
    context::AudioContext,
    media_element_source_node::MediaElementSourceNodeMessage,
    node::{AudioNodeInit, AudioNodeMessage},
};
use thread::JoinHandle;

struct PlayerContextDummy();
impl PlayerGLContext for PlayerContextDummy {
    fn get_gl_context(&self) -> GlContext {
        return GlContext::Unknown;
    }

    fn get_native_display(&self) -> NativeDisplay {
        return NativeDisplay::Unknown;
    }

    fn get_gl_api(&self) -> GlApi {
        return GlApi::None;
    }
}

pub struct MediaElementNode {
    player: Arc<Mutex<dyn Player>>,
    ipc_receiver: Arc<Mutex<IpcReceiver<PlayerEvent>>>,
    filename: String,
    decode_handle: Option<JoinHandle<()>>,
    play_handle: Option<JoinHandle<()>>,
}

impl MediaElementNode {
    pub fn new(
        filename: &str,
        servo_media: Arc<ServoMedia>,
        context: Arc<Mutex<AudioContext>>,
    ) -> MediaElementNode {
        let (renderer_sender, renderer_receiver) = mpsc::channel();
        let context = context.lock().unwrap();
        let source_node =
            context.create_node(AudioNodeInit::MediaElementSourceNode, Default::default());
        context.message_node(
            source_node,
            AudioNodeMessage::MediaElementSourceNode(
                MediaElementSourceNodeMessage::GetAudioRenderer(renderer_sender),
            ),
        );
        let audio_renderer = renderer_receiver.recv().unwrap();
        let (sender, receiver) = ipc::channel().unwrap();
        let player = servo_media.create_player(
            &ClientContextId::build(1, 1),
            StreamType::Seekable,
            sender,
            None,
            Some(audio_renderer),
            Box::new(PlayerContextDummy()),
        );
        MediaElementNode {
            filename: filename.to_owned(),
            player,
            ipc_receiver: Arc::new(Mutex::new(receiver)),
            decode_handle: None,
            play_handle: None,
        }
    }

    pub fn stop(&mut self) {
        self.player.lock().unwrap().stop().unwrap();
        if let Some(decode_handle) = self.decode_handle.take() {
            decode_handle.join();
        }

        if let Some(play_handle) = self.play_handle.take() {
            play_handle.join();
        }
    }

    pub fn seek(&self, time: f64) {
        self.player.lock().unwrap().seek(time).unwrap();
    }

    pub fn play(&mut self) {
        let path = Path::new(&self.filename);
        let display = path.display();

        let file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        if let Ok(metadata) = file.metadata() {
            self.player
                .lock()
                .unwrap()
                .set_input_size(metadata.len())
                .unwrap();
        }

        let player_clone = Arc::clone(&self.player);
        //let shutdown = Arc::new(AtomicBool::new(false));
        //let shutdown_clone = shutdown.clone();
        let (seek_sender, seek_receiver) = mpsc::channel();
        self.decode_handle = Some(thread::spawn(move || {
            let player = &player_clone;
            let mut buf_reader = BufReader::new(file);
            let mut buffer = [0; 1024];
            let mut read = |offset| {
                if buf_reader.seek(SeekFrom::Start(offset)).is_err() {
                    eprintln!("BufReader - Could not seek to {:?}", offset);
                }

                loop {
                    match buf_reader.read(&mut buffer[..]) {
                        Ok(0) => {
                            println!("Finished pushing data");
                            break;
                        }
                        Ok(size) => {
                            player
                                .lock()
                                .unwrap()
                                .push_data(Vec::from(&buffer[0..size]))
                                .unwrap();
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            break;
                        }
                    }
                }
            };

            while let Ok(position) = seek_receiver.try_recv() {
                println!();
                read(position);
            }
        }));

        self.player.lock().unwrap().play().unwrap();
        seek_sender.send(0).unwrap();
        let ipc_receiver = self.ipc_receiver.clone();
        self.play_handle = Some(thread::spawn(move || {
            while let Ok(event) = ipc_receiver.lock().unwrap().recv() {
                match event {
                    PlayerEvent::EndOfStream => {
                        println!("\nEOF");
                        break;
                    }
                    PlayerEvent::Error(ref s) => {
                        println!("\nError {:?}", s);
                        break;
                    }
                    PlayerEvent::MetadataUpdated(ref m) => {
                        println!("\nMetadata updated! {:?}", m);
                    }
                    PlayerEvent::StateChanged(ref s) => {
                        println!("\nPlayer state changed to {:?}", s);
                    }
                    PlayerEvent::VideoFrameUpdated => {}
                    PlayerEvent::PositionChanged(_) => println!("."),
                    PlayerEvent::SeekData(p, seek_lock) => {
                        println!("\nSeek requested to position {:?}", p);
                        seek_sender.send(p).unwrap();
                        seek_lock.unlock(true);
                    }
                    PlayerEvent::SeekDone(p) => println!("\nSeeked to {:?}", p),
                    PlayerEvent::NeedData => println!("\nNeedData"),
                    PlayerEvent::EnoughData => println!("\nEnoughData"),
                }
            }
        }));

        //let _ = context.resume();
    }
}
