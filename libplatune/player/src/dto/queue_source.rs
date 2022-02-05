use crate::{settings::Settings, source::Source};

#[derive(Debug)]
pub(crate) struct QueueSource {
    pub(crate) source: Box<dyn Source>,
    pub(crate) volume: Option<f64>,
    pub(crate) settings: Settings,
    pub(crate) force_restart_output: bool,
}
