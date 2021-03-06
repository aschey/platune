use crate::util::get_filename_from_path;
use crate::{context::CONTEXT, player_backend::PlayerBackend};
use act_zero::*;
use log::info;
use postage::broadcast::Sender;
use servo_media_audio::{
    block::Block,
    buffer_source_node::{AudioBuffer, AudioBufferSourceNodeMessage},
    gain_node::GainNodeOptions,
    graph::NodeId,
    node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage, OnEndedCallback},
};

use super::decoder::Decoder;
pub struct Player {
    player_backend: Box<PlayerBackend>,
    decoder: Addr<Decoder>,
    sources: Vec<ScheduledSource>,
    volume: f32,
    ended_tx: Sender<String>,
}

impl Actor for Player {}

impl Player {
    pub fn new(
        player_backend: Box<PlayerBackend>,
        decoder: Addr<Decoder>,
        ended_tx: Sender<String>,
    ) -> Player {
        Player {
            player_backend,
            decoder,
            sources: vec![],
            volume: 0.5,
            ended_tx,
        }
    }
    // fn play(&self, start_time: f64) {
    //     self.player_backend
    //         .play(self.sources[0].buffer_source, start_time);
    // }

    pub async fn pause(&mut self) {
        let context = CONTEXT.lock().unwrap();
        self.player_backend.pause(&context);
    }

    pub async fn resume(&mut self) {
        let context = CONTEXT.lock().unwrap();
        self.player_backend.resume(&context);
    }

    pub async fn stop(&mut self) {
        let context = CONTEXT.lock().unwrap();
        self.player_backend
            .stop(&context, self.sources[0].buffer_source);
    }

    pub async fn seek(&mut self, seconds: f64) {
        self.stop().await;

        let queued_songs = self
            .sources
            .iter()
            .map(|s| s.path.to_owned())
            .collect::<Vec<_>>();

        self.disconnect_all();
        self.sources = vec![];
        self.load(queued_songs.get(0).unwrap().to_owned(), Some(seconds))
            .await;
        if queued_songs.len() > 1 {
            self.load(queued_songs.get(1).unwrap().to_owned(), None)
                .await;
        }
    }

    pub async fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        let context = CONTEXT.lock().unwrap();
        for source in &self.sources {
            self.player_backend
                .set_volume(&context, source.gain, volume);
        }
    }

    pub async fn load(&mut self, path: String, start_seconds: Option<f64>) {
        let file = get_filename_from_path(&path);
        info!("Loading {}", file);
        let file_info = call!(self.decoder.decode(path.to_owned())).await.unwrap();

        let context = CONTEXT.lock().unwrap();

        let buffer_source = context.create_node(
            AudioNodeInit::AudioBufferSourceNode(Default::default()),
            Default::default(),
        );

        let gain = context.create_node(
            AudioNodeInit::GainNode(GainNodeOptions { gain: self.volume }),
            Default::default(),
        );

        let analyser = context.create_node(
            AudioNodeInit::AnalyserNode(Box::new(move |mut block| {
                let data = block.data_mut();
                let json = serde_json::to_string(data).unwrap();
                //println!("{:?}", json);
            })),
            Default::default(),
        );

        let dest = context.dest_node();

        context.connect_ports(buffer_source.output(0), analyser.input(0));
        context.connect_ports(buffer_source.output(0), gain.input(0));
        context.connect_ports(gain.output(0), dest.input(0));
        context.connect_ports(analyser.output(0), dest.input(0));

        // let callback = OnEndedCallback::new(|| {
        //     info!("Playback ended");
        // });

        context.message_node(
            buffer_source,
            AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetBuffer(Some(
                AudioBuffer::from_buffers(file_info.data, file_info.sample_rate as f32),
            ))),
        );

        let start_time: f64;

        self.player_backend.subscribe_onended(
            &context,
            buffer_source,
            file.to_owned(),
            self.ended_tx.clone(),
        );

        // context.message_node(
        //     buffer_source,
        //     AudioNodeMessage::AudioScheduledSourceNode(
        //         AudioScheduledSourceNodeMessage::RegisterOnEndedCallback(callback),
        //     ),
        // );

        if self.sources.len() == 0 {
            if let Some(seconds) = start_seconds {
                info!("Seeking to {}", seconds);
                self.player_backend.seek(&context, buffer_source, seconds);
            }

            start_time = context.current_time();
            self.player_backend
                .play(&context, buffer_source, start_time);
            info!("Starting at {}", start_time);
        } else {
            let prev = self.sources.last().unwrap();
            let seconds =
                prev.start_time + (prev.duration - prev.end_gap - file_info.start_gap) - 0.03;
            start_time = seconds;
            info!("Starting at {}", seconds);
            self.player_backend.play(&context, buffer_source, seconds);
        }

        let gap = start_seconds.unwrap_or_default();

        info!(
            "Adding {} start time: {} start gap: {} end gap: {} duration: {} gap: {} computed duration: {}",
            file,
            start_time,
            file_info.start_gap,
            file_info.end_gap,
            file_info.duration,
            gap,
            file_info.duration - gap
        );
        self.sources.push(ScheduledSource {
            path,
            start_time,
            start_gap: file_info.start_gap,
            end_gap: file_info.end_gap,
            duration: file_info.duration - gap,
            buffer_source,
            gain,
            analyser,
        });
    }

    fn disconnect_all(&mut self) {
        let context = CONTEXT.lock().unwrap();
        for source in &self.sources {
            context.disconnect_all_from(source.buffer_source);
            context.disconnect_all_from(source.gain);
            context.disconnect_all_from(source.analyser);
        }
    }
}

struct ScheduledSource {
    path: String,
    start_time: f64,
    start_gap: f64,
    end_gap: f64,
    duration: f64,
    buffer_source: NodeId,
    gain: NodeId,
    analyser: NodeId,
}
