use crate::libplayer::PlayerEvent;
use crate::util::get_filename_from_path;
use crate::{context::CONTEXT, player_backend::PlayerBackend};
use act_zero::*;
use log::{error, info, warn};
use postage::{
    broadcast::{self, Sender},
    mpsc,
    sink::Sink,
};
use servo_media_audio::{
    analyser_node::AnalysisEngine,
    block::Block,
    buffer_source_node::{AudioBuffer, AudioBufferSourceNodeMessage},
    context::AudioContext,
    gain_node::GainNodeOptions,
    graph::NodeId,
    node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage, OnEndedCallback},
};
use std::{collections::VecDeque, fmt::Debug, sync::Arc};

use super::{decoder::Decoder, request_handler::Command};
pub struct Player {
    player_backend: Box<PlayerBackend>,
    decoder: Addr<Decoder>,
    sources: VecDeque<ScheduledSource>,
    volume: f32,
    event_tx: broadcast::Sender<PlayerEvent>,
    analysis_tx: mpsc::Sender<Block>,
    request_queue: mpsc::Sender<Command>,
    should_load_next: bool,
}

impl Actor for Player {}

impl Player {
    pub fn new(
        player_backend: Box<PlayerBackend>,
        decoder: Addr<Decoder>,
        event_tx: broadcast::Sender<PlayerEvent>,
        analysis_tx: mpsc::Sender<Block>,
        request_queue: mpsc::Sender<Command>,
    ) -> Player {
        Player {
            player_backend,
            decoder,
            sources: VecDeque::new(),
            volume: -0.5,
            event_tx,
            analysis_tx,
            request_queue,
            should_load_next: true,
        }
    }
    // fn play(&self, start_time: f64) {
    //     self.player_backend
    //         .play(self.sources[0].buffer_source, start_time);
    // }

    pub async fn pause(&mut self) {
        let context = CONTEXT.lock().unwrap();
        self.player_backend.pause(&context);
        self.event_tx.publish(PlayerEvent::Pause {
            file: self.current_file(),
        });
    }

    pub async fn ensure_resumed(&self) {
        let context = CONTEXT.lock().unwrap();
        self.player_backend.resume(&context);
    }

    pub async fn resume(&mut self) {
        self.ensure_resumed().await;
        self.event_tx.publish(PlayerEvent::Resume {
            file: self.current_file(),
        });
    }

    pub async fn reset(&mut self) {
        let context = CONTEXT.lock().unwrap();
        if let Some(current_source) = self.sources.get(0) {
            self.player_backend
                .stop(&context, current_source.buffer_source);
        }

        self.disconnect_all(&context);
        self.sources = VecDeque::new();
        self.should_load_next = false;
    }

    pub async fn stop(&mut self) {
        self.reset().await;

        self.event_tx.publish(PlayerEvent::Stop {
            file: self.current_file(),
        });
    }

    pub async fn should_load_next(&self) -> ActorResult<bool> {
        Produces::ok(self.should_load_next)
    }

    pub async fn seek(&mut self, seconds: f64) {
        let queued_songs = self
            .sources
            .iter()
            .map(|s| s.path.to_owned())
            .collect::<Vec<_>>();

        self.reset().await;

        if let Some(first) = queued_songs.first() {
            self.load(first.to_owned(), Some(seconds)).await;
        }
        if let Some(next) = queued_songs.get(1) {
            self.load(next.to_owned(), None).await;
        }

        self.event_tx.publish(PlayerEvent::Seek {
            file: self.current_file().to_owned(),
            time: seconds,
        });
    }

    pub async fn on_ended(&mut self) {
        self.sources.drain(0..1);
        info!(
            "Player event: ended. Scheduled song count: {}",
            self.sources.len()
        );
    }

    pub async fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
        let context = CONTEXT.lock().unwrap();
        for source in &self.sources {
            self.player_backend
                .set_volume(&context, source.gain, volume);
        }
        self.event_tx.publish(PlayerEvent::SetVolume {
            file: self.current_file(),
            volume,
        });
    }

    pub async fn load(&mut self, path: String, start_seconds: Option<f64>) {
        self.should_load_next = true;
        let file = get_filename_from_path(&path);
        info!(
            "Loading {}. Scheduled song count: {}",
            file,
            self.sources.len()
        );
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

        let mut tx = self.analysis_tx.clone();
        let analyser = context.create_node(
            AudioNodeInit::AnalyserNode(Box::new(move |block| {
                tx.try_send(block).unwrap_or_default();
            })),
            Default::default(),
        );

        let dest = context.dest_node();

        context.connect_ports(buffer_source.output(0), analyser.input(0));
        context.connect_ports(buffer_source.output(0), gain.input(0));
        context.connect_ports(gain.output(0), dest.input(0));
        context.connect_ports(analyser.output(0), dest.input(0));

        context.message_node(
            buffer_source,
            AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetBuffer(Some(
                AudioBuffer::from_buffers(file_info.data, file_info.sample_rate as f32),
            ))),
        );

        let start_time: f64;
        let mut sender = self.event_tx.clone();
        let mut request_queue = self.request_queue.clone();
        let file_ = file.clone();

        self.player_backend.subscribe_onended(
            &context,
            buffer_source,
            Box::new(move || {
                info!("{:?} ended", file_);
                request_queue.try_send(Command::Ended).unwrap();
                sender.publish(PlayerEvent::Ended { file: file_ });
            }),
        );

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
            let prev = self.sources.back().unwrap();
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
        self.sources.push_back(ScheduledSource {
            path,
            start_time,
            end_gap: file_info.end_gap,
            duration: file_info.duration - gap,
            buffer_source,
            gain,
            analyser,
        });
    }

    fn disconnect_all(&mut self, context: &AudioContext) {
        for source in &self.sources {
            context.disconnect_all_from(source.buffer_source);
            context.disconnect_all_from(source.gain);
            context.disconnect_all_from(source.analyser);
        }
    }

    fn current_file(&self) -> String {
        if let Some(source) = self.sources.front() {
            return source.path.to_owned();
        }
        return "".to_owned();
    }
}

struct ScheduledSource {
    path: String,
    start_time: f64,
    end_gap: f64,
    duration: f64,
    buffer_source: NodeId,
    gain: NodeId,
    analyser: NodeId,
}

pub trait SenderExt<T> {
    fn publish(&mut self, value: T);
}

impl<T> SenderExt<T> for Sender<T>
where
    T: Clone + Debug,
{
    fn publish(&mut self, value: T) {
        if let Err(res) = self.try_send(value) {
            warn!("Unable to send: {:?}", res);
        }
    }
}
