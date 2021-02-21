use std::sync::{Arc, Mutex};

use servo_media::{ClientContextId, ServoMedia};
use servo_media_audio::{
    context::{AudioContext, AudioContextOptions, RealTimeAudioContextOptions},
    graph::NodeId,
    node::{AudioNodeMessage, AudioScheduledSourceNodeMessage},
};

pub struct ServoBackend {
    context: Arc<Mutex<AudioContext>>,
}

impl ServoBackend {
    pub fn new() -> ServoBackend {
        ServoMedia::init::<servo_media_auto::Backend>();
        let servo_media = ServoMedia::get().unwrap();
        let options = <RealTimeAudioContextOptions>::default();

        let context = servo_media.create_audio_context(
            &ClientContextId::build(1, 1),
            AudioContextOptions::RealTimeAudioContext(options),
        );

        ServoBackend { context }
    }

    pub fn play(&self, node_id: NodeId, start_time: f64) {
        self.context.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(
                start_time,
            )),
        );
    }
}
