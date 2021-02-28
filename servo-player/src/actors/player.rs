use crate::context::CONTEXT;
use act_zero::*;
use gstreamer::ClockTime;
use log::info;
use servo_media_audio::{
    buffer_source_node::{AudioBuffer, AudioBufferSourceNodeMessage},
    graph::NodeId,
    node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage, OnEndedCallback},
};

use crate::player_backend::PlayerBackend;

use super::decoder::Decoder;
pub struct Player<T: PlayerBackend + Send + 'static> {
    player_backend: T,
    decoder: Addr<Decoder>,
    sources: Vec<ScheduledSource>,
}

impl<T: PlayerBackend + Send + 'static> Actor for Player<T> {}

impl<T: PlayerBackend + Send + 'static> Player<T> {
    pub fn new(player_backend: T, decoder: Addr<Decoder>) -> Player<T> {
        Player {
            player_backend,
            decoder,
            sources: vec![],
        }
    }
    // fn play(&self, start_time: f64) {
    //     self.player_backend
    //         .play(self.sources[0].buffer_source, start_time);
    // }

    pub async fn pause(&self) {
        self.player_backend.pause();
    }

    pub async fn stop(&self) {
        self.player_backend.stop(self.sources[0].buffer_source);
    }

    pub async fn load(&mut self, file: String) {
        let file_info = call!(self.decoder.decode(file)).await.unwrap();

        let context = CONTEXT.lock().unwrap();

        let buffer_source = context.create_node(
            AudioNodeInit::AudioBufferSourceNode(Default::default()),
            Default::default(),
        );

        let dest = context.dest_node();
        context.connect_ports(buffer_source.output(0), dest.input(0));
        let callback = OnEndedCallback::new(|| {
            info!("Playback ended");
        });

        context.message_node(
            buffer_source,
            AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetBuffer(Some(
                AudioBuffer::from_buffers(file_info.data, file_info.sample_rate as f32),
            ))),
        );

        context.message_node(
            buffer_source,
            AudioNodeMessage::AudioScheduledSourceNode(
                AudioScheduledSourceNodeMessage::RegisterOnEndedCallback(callback),
            ),
        );
        drop(context);

        if self.sources.len() == 0 {
            self.player_backend.play(buffer_source, 0.);
        } else {
            let prev = self.sources.last().unwrap();
            let start_time =
                prev.duration.nseconds().unwrap() as f64 / 1e9 - prev.end_gap - file_info.start_gap;
            self.player_backend.play(buffer_source, start_time);
        }

        self.sources.push(ScheduledSource {
            start_gap: file_info.start_gap,
            end_gap: file_info.end_gap,
            duration: file_info.duration,
            buffer_source,
        });
    }
}

struct ScheduledSource {
    start_gap: f64,
    end_gap: f64,
    duration: ClockTime,
    buffer_source: NodeId,
}
