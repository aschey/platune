use crate::libplayer::PlayerEvent;
use crate::util::get_filename_from_path;
use act_zero::*;
use log::{error, info, warn};
use postage::{
    broadcast::{self, Sender},
    mpsc,
    sink::Sink,
};
use servo_media::{ClientContextId, ServoMedia};
use servo_media_audio::{
    analyser_node::AnalysisEngine,
    block::Block,
    buffer_source_node::{AudioBuffer, AudioBufferSourceNodeMessage},
    context::{AudioContext, AudioContextOptions, RealTimeAudioContextOptions},
    gain_node::GainNodeOptions,
    graph::NodeId,
    node::{AudioNodeInit, AudioNodeMessage, AudioScheduledSourceNodeMessage, OnEndedCallback},
};
use std::{
    cmp::max,
    collections::VecDeque,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use super::{
    decoder::Decoder,
    gstreamer_context::{GStreamerContext, NodeGroup},
    request_handler::Command,
};
pub struct Player {
    decoder: Addr<Decoder>,
    context: Addr<GStreamerContext>,
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
        decoder: Addr<Decoder>,
        context: Addr<GStreamerContext>,
        event_tx: broadcast::Sender<PlayerEvent>,
        analysis_tx: mpsc::Sender<Block>,
        request_queue: mpsc::Sender<Command>,
    ) -> Player {
        Player {
            decoder,
            context,
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
        call!(self.context.pause()).await.unwrap();
        self.event_tx.publish(PlayerEvent::Pause);
    }

    pub async fn ensure_resumed(&self) {
        call!(self.context.resume()).await.unwrap();
    }

    pub async fn resume(&mut self) {
        self.ensure_resumed().await;
        self.event_tx.publish(PlayerEvent::Resume);
    }

    pub async fn reset(&mut self) {
        self.disconnect_all().await;
        if let Some(current_source) = self.sources.get(0) {
            call!(self.context.stop(current_source.nodes.buffer_source));
        }

        self.sources = VecDeque::new();
        self.should_load_next = false;
    }

    pub async fn stop(&mut self) {
        self.reset().await;

        self.event_tx.publish(PlayerEvent::Stop);
    }

    pub async fn shutdown(&self) {
        call!(self.context.close()).await.unwrap();
    }

    pub async fn should_load_next(&mut self) -> ActorResult<bool> {
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

        let gains = self.sources.iter().map(|s| s.nodes.gain).collect();
        call!(self.context.set_volume(gains, volume)).await.unwrap();

        self.event_tx.publish(PlayerEvent::SetVolume { volume });
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

        let buffer_source_init = AudioNodeInit::AudioBufferSourceNode(Default::default());
        let gain_init = AudioNodeInit::GainNode(GainNodeOptions { gain: self.volume });
        let mut tx = self.analysis_tx.clone();
        let analyser_init = AudioNodeInit::AnalyserNode(Box::new(move |block| {
            tx.try_send(block).unwrap_or_default();
        }));

        let set_buffer =
            AudioNodeMessage::AudioBufferSourceNode(AudioBufferSourceNodeMessage::SetBuffer(Some(
                AudioBuffer::from_buffers(file_info.data, file_info.sample_rate as f32),
            )));

        let nodes = call!(self.context.create_nodes(
            buffer_source_init,
            gain_init,
            analyser_init,
            set_buffer
        ))
        .await
        .unwrap();

        let buffer_source = nodes.buffer_source;

        let start_time: f64;
        let mut sender = self.event_tx.clone();
        let mut request_queue = self.request_queue.clone();
        let file_ = file.clone();

        call!(self.context.subscribe_onended(
            buffer_source,
            Box::new(move || {
                info!("{:?} ended", file_);
                sender.publish(PlayerEvent::Ended);
                request_queue.try_send(Command::Ended).unwrap();
            })
        ));
        let seek_start = start_seconds.unwrap_or_default();
        if self.sources.len() == 0 {
            if let Some(seconds) = start_seconds {
                info!("Seeking to {}", seconds);
                call!(self.context.seek(buffer_source, seconds))
                    .await
                    .unwrap();
            }

            start_time = call!(self.context.current_time()).await.unwrap();
            call!(self.context.play(buffer_source, start_time))
                .await
                .unwrap();
            info!("Starting at {}", start_time);
            if start_seconds == None {
            } else {
                self.event_tx
                    .publish(PlayerEvent::Seek { time: seek_start });
            }
        } else {
            let prev = self.sources.back().unwrap();
            let seconds =
                prev.start_time + (prev.duration - prev.end_gap - file_info.start_gap) - 0.03;
            start_time = seconds;
            info!("Starting at {}", seconds);
            call!(self.context.play(buffer_source, seconds))
                .await
                .unwrap();
        }

        let seek_start = start_seconds.unwrap_or_default();
        let computed_duration = f64::max(file_info.duration - seek_start, 0.);

        info!(
            "Adding {} start time: {} start gap: {} end gap: {} duration: {} seek_start: {} computed duration: {}",
            file,
            start_time,
            file_info.start_gap,
            file_info.end_gap,
            file_info.duration,
            seek_start,
            computed_duration
        );
        self.sources.push_back(ScheduledSource {
            path,
            start_time,
            end_gap: file_info.end_gap,
            duration: computed_duration,
            nodes,
        });
    }

    async fn disconnect_all(&mut self) {
        let all_nodes = self
            .sources
            .iter()
            .map(|s| s.nodes.to_vec())
            .flatten()
            .collect();
        call!(self.context.disconnect_all(all_nodes)).await.unwrap();
    }

    fn current_file_path(&self) -> String {
        if let Some(source) = self.sources.front() {
            return source.path.to_owned();
        }
        return "".to_owned();
    }

    fn current_file_name(&self) -> String {
        get_filename_from_path(&self.current_file_path())
    }
}

struct ScheduledSource {
    path: String,
    start_time: f64,
    end_gap: f64,
    duration: f64,
    nodes: NodeGroup,
}

pub trait SenderExt<T> {
    fn publish(&mut self, value: T);
}

impl<T> SenderExt<T> for Sender<T>
where
    T: Clone + Debug,
{
    fn publish(&mut self, value: T) {
        info!("Publishing {:?}", value);
        if let Err(res) = self.try_send(value) {
            warn!("Unable to send: {:?}", res);
        }
    }
}
