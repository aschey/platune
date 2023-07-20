use crate::settings::Settings;
use decal::decoder::Source;

#[derive(Debug)]
pub(crate) struct QueueSource {
    pub(crate) source: Box<dyn Source>,
    pub(crate) volume: Option<f32>,
    pub(crate) settings: Settings,
    pub(crate) queue_start_mode: QueueStartMode,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum QueueStartMode {
    ForceRestart {
        device_name: Option<String>,
        paused: bool,
    },
    Normal,
}
