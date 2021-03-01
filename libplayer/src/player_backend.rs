use servo_media_audio::graph::NodeId;

pub trait PlayerBackendImpl {
    fn play(&self, node_id: NodeId, start_time: f64);
    fn pause(&self);
    fn stop(&self, node_id: NodeId);
    fn seek(&self, node_id: NodeId, time: f64);
    fn set_volume(&self, node_id: NodeId, value: f32);
}

pub type PlayerBackend = dyn PlayerBackendImpl + Send + 'static;
