use std::sync::{Arc, Mutex};

use servo_media_audio::{
    context::AudioContext,
    graph::NodeId,
    node::{AudioNodeMessage, AudioScheduledSourceNodeMessage},
};

use crate::media_element_node::MediaElementNode;

pub struct AudioNodeWrapper {
    media_element_node: Option<MediaElementNode>,
    node_id: NodeId,
    context: Arc<Mutex<AudioContext>>,
}

impl AudioNodeWrapper {
    pub fn start(&mut self, start_time: Option<f64>) {
        match self.media_element_node.as_mut() {
            Some(node) => node.play(),
            None => {
                self.context.lock().unwrap().message_node(
                    self.node_id,
                    AudioNodeMessage::AudioScheduledSourceNode(
                        AudioScheduledSourceNodeMessage::Start(start_time.unwrap_or(0.0)),
                    ),
                );
            }
        }
    }

    pub fn stop(&mut self, stop_time: Option<f64>) {
        match self.media_element_node.as_mut() {
            Some(node) => node.stop(),
            None => self.context.lock().unwrap().message_node(
                self.node_id,
                AudioNodeMessage::AudioScheduledSourceNode(AudioScheduledSourceNodeMessage::Stop(
                    stop_time.unwrap_or(0.0),
                )),
            ),
        }
    }
}
