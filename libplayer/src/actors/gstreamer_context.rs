use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use act_zero::{Actor, ActorResult, Produces};
use servo_media::{BackendInit, ClientContextId, ServoMedia};
use servo_media_audio::{
    block::Block,
    buffer_source_node::AudioBufferSourceNodeMessage,
    context::{AudioContext, AudioContextOptions, RealTimeAudioContextOptions},
    decoder::AudioDecoderCallbacks,
    gain_node::GainNodeOptions,
    graph::NodeId,
    node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage, OnEndedCallback},
    param::{ParamType, UserAutomationEvent},
};

pub struct GStreamerContext {
    context: Arc<Mutex<AudioContext>>,
}

impl Actor for GStreamerContext {}

impl GStreamerContext {
    pub fn new<T: BackendInit>() -> GStreamerContext {
        ServoMedia::init::<T>();
        let servo_media = ServoMedia::get().unwrap();
        let context = servo_media.create_audio_context(
            &ClientContextId::build(1, 1),
            AudioContextOptions::RealTimeAudioContext(RealTimeAudioContextOptions::default()),
        );
        context.lock().unwrap().resume().unwrap();
        GStreamerContext { context }
    }

    pub async fn play(&self, node_id: NodeId, start_seconds: f64) {
        self.context.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Start(
                start_seconds,
            )),
        );
    }

    pub async fn pause(&self) {
        self.context.lock().unwrap().suspend().unwrap();
    }

    pub async fn resume(&self) {
        self.context.lock().unwrap().resume().unwrap();
    }

    pub async fn stop(&self, node_id: NodeId) {
        self.context.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Stop(0.)),
        );
    }

    pub async fn seek(&self, node_id: NodeId, seconds: f64) {
        self.context.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetStartParams(
                0.,
                Some(seconds),
                None,
            )),
        );
    }

    pub async fn set_volume(&self, node_ids: Vec<NodeId>, value: f32) {
        let context = self.context.lock().unwrap();
        for node_id in node_ids {
            context.message_node(
                node_id,
                AudioNodeMessage::SetParam(ParamType::Gain, UserAutomationEvent::SetValue(value)),
            );
        }
    }

    pub async fn subscribe_onended(
        &self,
        node_id: NodeId,
        callback: Box<dyn FnOnce() + Send + 'static>,
    ) {
        self.context.lock().unwrap().message_node(
            node_id,
            AudioNodeMessage::AudioScheduledSourceNode(
                AudioScheduledSourceNodeMessage::RegisterOnEndedCallback(OnEndedCallback::new(
                    callback,
                )),
            ),
        );
    }

    pub async fn decode_audio_data(&self, bytes: Vec<u8>, callbacks: AudioDecoderCallbacks) {
        self.context
            .lock()
            .unwrap()
            .decode_audio_data(bytes, callbacks);
    }

    pub async fn disconnect_all(&self, sources: Vec<NodeId>) {
        let context = self.context.lock().unwrap();
        for source in sources {
            context.disconnect_all_from(source);
        }
    }

    pub async fn create_nodes(
        &self,
        buffer_source_init: AudioNodeInit,
        gain_init: AudioNodeInit,
        analyser_init: AudioNodeInit,
        set_buffer: AudioNodeMessage,
    ) -> ActorResult<NodeGroup> {
        let context = self.context.lock().unwrap();

        let buffer_source = context.create_node(buffer_source_init, Default::default());
        let gain = context.create_node(gain_init, Default::default());
        let analyser = context.create_node(analyser_init, Default::default());
        let dest = context.dest_node();

        context.connect_ports(buffer_source.output(0), analyser.input(0));
        context.connect_ports(buffer_source.output(0), gain.input(0));
        context.connect_ports(gain.output(0), dest.input(0));
        context.connect_ports(analyser.output(0), dest.input(0));

        context.message_node(buffer_source, set_buffer);

        Produces::ok(NodeGroup {
            buffer_source,
            gain,
            analyser,
        })
    }

    pub async fn current_time(&self) -> ActorResult<f64> {
        Produces::ok(self.context.lock().unwrap().current_time())
    }

    pub async fn close(&self) {
        self.context.lock().unwrap().close().unwrap();
    }
}

pub struct NodeGroup {
    pub buffer_source: NodeId,
    pub gain: NodeId,
    pub analyser: NodeId,
}

impl NodeGroup {
    pub fn to_vec(&self) -> Vec<NodeId> {
        vec![self.buffer_source, self.gain, self.analyser]
    }
}
