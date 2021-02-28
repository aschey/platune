use crate::context::CONTEXT;
use servo_media::{ClientContextId, ServoMedia};
use servo_media_audio::{
    context::{AudioContext, AudioContextOptions, RealTimeAudioContextOptions},
    graph::NodeId,
    node::{AudioNodeMessage, AudioScheduledSourceNodeMessage},
};
use std::sync::{Arc, Mutex};

use crate::player_backend::PlayerBackend;

pub struct ServoBackend {}

impl PlayerBackend for ServoBackend {
    fn play(&self, node_id: NodeId, start_time: f64) {
        CONTEXT.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(
                start_time,
            )),
        );
    }

    fn pause(&self) {
        CONTEXT.lock().unwrap().suspend().unwrap();
    }

    fn stop(&self, node_id: NodeId) {
        CONTEXT.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Stop(0.)),
        );
    }
}
