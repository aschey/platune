use act_zero::*;
use servo_media_audio::graph::NodeId;

use crate::player_backend::PlayerBackend;
struct Player<T: PlayerBackend + Send + 'static> {
    player_backend: T,
}

impl<T: PlayerBackend + Send + 'static> Actor for Player<T> {}

impl<T: PlayerBackend + Send + 'static> Player<T> {
    pub fn play(&self, node_id: NodeId, start_time: f64) {
        self.player_backend.play(node_id, start_time);
    }

    pub fn pause(&self) {
        self.player_backend.pause();
    }

    pub fn stop(&self, node_id: NodeId) {
        self.player_backend.stop(node_id);
    }
}
