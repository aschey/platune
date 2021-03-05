use postage::broadcast::Sender;
use servo_media_audio::{context::AudioContext, graph::NodeId};

pub trait PlayerBackendImpl {
    fn play(&self, context: &AudioContext, node_id: NodeId, start_seconds: f64);
    fn pause(&self, context: &AudioContext);
    fn resume(&self, context: &AudioContext);
    fn stop(&self, context: &AudioContext, node_id: NodeId);
    fn seek(&self, context: &AudioContext, node_id: NodeId, seconds: f64);
    fn set_volume(&self, context: &AudioContext, node_id: NodeId, value: f32);
    fn subscribe_onended(
        &self,
        context: &AudioContext,
        node_id: NodeId,
        file: String,
        sender: Sender<String>,
    );
}

pub type PlayerBackend = dyn PlayerBackendImpl + Send + 'static;
