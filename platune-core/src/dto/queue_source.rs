use crate::{settings::resample_mode::ResampleMode, source::Source};

pub(crate) struct QueueSource {
    pub(crate) source: Box<dyn Source>,
    pub(crate) resample_mode: ResampleMode,
    pub(crate) force_restart_output: bool,
}
