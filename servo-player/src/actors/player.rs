use crate::context::CONTEXT;
use act_zero::*;
use gstreamer::ClockTime;
use log::info;
use servo_media_audio::{
    block::Block,
    buffer_source_node::{AudioBuffer, AudioBufferSourceNodeMessage},
    gain_node::GainNodeOptions,
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

    pub async fn pause(&mut self) {
        self.player_backend.pause();
    }

    pub async fn stop(&mut self) {
        self.player_backend.stop(self.sources[0].buffer_source);
    }

    pub async fn seek(&mut self, time: f64) {
        self.stop().await;

        let source = self.sources[0].file.to_owned();
        self.sources = vec![];
        self.load(source, Some(time)).await;
    }

    pub async fn load(&mut self, file: String, start_time: Option<f64>) {
        let file_info = call!(self.decoder.decode(file.to_owned())).await.unwrap();

        let context = CONTEXT.lock().unwrap();

        let buffer_source = context.create_node(
            AudioNodeInit::AudioBufferSourceNode(Default::default()),
            Default::default(),
        );

        let gain = context.create_node(
            AudioNodeInit::GainNode(GainNodeOptions { gain: 0.2 }),
            Default::default(),
        );

        let analyser = context.create_node(
            AudioNodeInit::AnalyserNode(Box::new(move |block| {})),
            Default::default(),
        );

        let dest = context.dest_node();

        context.connect_ports(buffer_source.output(0), analyser.input(0));
        context.connect_ports(buffer_source.output(0), gain.input(0));
        context.connect_ports(gain.output(0), dest.input(0));
        context.connect_ports(analyser.output(0), dest.input(0));
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

        if self.sources.len() == 0 {
            drop(context);
            if let Some(start) = start_time {
                self.player_backend.seek(buffer_source, start);
            }

            self.player_backend.play(buffer_source, 0.);
        } else {
            let prev = self.sources.last().unwrap();
            let start_time = prev.duration.nseconds().unwrap() as f64 / 1e9
                - prev.end_gap
                - file_info.start_gap
                - context.current_time() / 1000.;
            drop(context);
            self.player_backend.play(buffer_source, start_time);
        }

        self.sources.push(ScheduledSource {
            file,
            start_gap: file_info.start_gap,
            end_gap: file_info.end_gap,
            duration: file_info.duration,
            buffer_source,
            gain,
            analyser,
        });
    }
}

struct ScheduledSource {
    file: String,
    start_gap: f64,
    end_gap: f64,
    duration: ClockTime,
    buffer_source: NodeId,
    gain: NodeId,
    analyser: NodeId,
}
