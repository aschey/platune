use crate::context::CONTEXT;
use log::{info, warn};
use postage::{broadcast::Sender, sink::Sink};
use servo_media_audio::{
    buffer_source_node::AudioBufferSourceNodeMessage,
    context::AudioContext,
    graph::NodeId,
    node::{AudioNodeMessage, AudioScheduledSourceNodeMessage, OnEndedCallback},
    param::{ParamType, UserAutomationEvent},
};

use crate::player_backend::PlayerBackendImpl;

pub struct ServoBackend {}

impl PlayerBackendImpl for ServoBackend {
    fn play(&self, context: &AudioContext, node_id: NodeId, start_seconds: f64) {
        context.message_node(
            node_id,
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(
                start_seconds,
            )),
        );
    }

    fn pause(&self, context: &AudioContext) {
        context.suspend().unwrap();
    }

    fn resume(&self, context: &AudioContext) {
        context.resume().unwrap();
    }

    fn stop(&self, context: &AudioContext, node_id: NodeId) {
        context.message_node(
            node_id,
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Stop(0.)),
        );
    }

    fn seek(&self, context: &AudioContext, node_id: NodeId, seconds: f64) {
        context.message_node(
            node_id,
            AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetStartParams(
                0.,
                Some(seconds),
                None,
            )),
        );
    }

    fn set_volume(&self, context: &AudioContext, node_id: NodeId, value: f32) {
        context.message_node(
            node_id,
            AudioNodeMessage::SetParam(ParamType::Gain, UserAutomationEvent::SetValue(value)),
        );
    }

    fn subscribe_onended(
        &self,
        context: &AudioContext,
        node_id: NodeId,
        file: String,
        mut sender: Sender<String>,
    ) {
        context.message_node(
            node_id,
            AudioNodeMessage::AudioScheduledSourceNode(
                AudioScheduledSourceNodeMessage::RegisterOnEndedCallback(OnEndedCallback::new(
                    move || {
                        info!("{:?} ended", file);
                        if let Err(res) = sender.try_send(file) {
                            warn!("{:?}", res);
                        }
                    },
                )),
            ),
        );
    }
}
