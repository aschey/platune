use crate::context::CONTEXT;
use servo_media_audio::{
    buffer_source_node::AudioBufferSourceNodeMessage,
    graph::NodeId,
    node::{AudioNodeMessage, AudioScheduledSourceNodeMessage},
    param::{ParamType, UserAutomationEvent},
};

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

    fn seek(&self, node_id: NodeId, time: f64) {
        CONTEXT.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetStartParams(
                0.,
                Some(time),
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
