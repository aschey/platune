use std::time::Duration;

use crate::{settings::Settings, source::Source};

#[derive(Debug)]
pub(crate) struct QueueSource {
    pub(crate) source: Box<dyn Source>,
    pub(crate) volume: Option<f64>,
    pub(crate) settings: Settings,
    pub(crate) queue_start_mode: QueueStartMode,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum QueueStartMode {
    ForceRestart { start_time: Option<Duration> },
    Normal,
}
