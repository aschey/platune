use servo_media_audio::graph::NodeId;

pub trait PlayerBackend {
    fn play(&self, node_id: NodeId, start_time: f64);
    fn pause(&self);
    fn stop(&self, node_id: NodeId);
    fn set_volume(&self, node_id: NodeId, value: f32);
}
