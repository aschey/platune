use crate::context::CONTEXT;
use servo_media_audio::{
    buffer_source_node::AudioBufferSourceNodeMessage,
    graph::NodeId,
    node::{AudioNodeMessage, AudioScheduledSourceNodeMessage},
    param::{ParamType, UserAutomationEvent},
};

use crate::player_backend::PlayerBackendImpl;

pub struct ServoBackend {}

impl PlayerBackendImpl for ServoBackend {
    fn play(&self, node_id: NodeId, start_seconds: f64) {
        CONTEXT.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(
                start_seconds,
            )),
        );
    }

    fn pause(&self) {
        CONTEXT.lock().unwrap().suspend().unwrap();
    }

    fn resume(&self) {
        CONTEXT.lock().unwrap().resume().unwrap();
    }

    fn stop(&self, node_id: NodeId) {
        CONTEXT.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Stop(0.)),
        );
    }

    fn seek(&self, node_id: NodeId, seconds: f64) {
        CONTEXT.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetStartParams(
                0.,
                Some(seconds),
                None,
            )),
        );
    }

    fn set_volume(&self, node_id: NodeId, value: f32) {
        CONTEXT.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::SetParam(ParamType::Gain, UserAutomationEvent::SetValue(value)),
        );
    }
}
