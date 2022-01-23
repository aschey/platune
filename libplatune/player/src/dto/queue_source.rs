use crate::{settings::Settings, source::Source};

pub(crate) struct QueueSource {
    pub(crate) source: Box<dyn Source>,
    pub(crate) settings: Settings,
    pub(crate) force_restart_output: bool,
}
