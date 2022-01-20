use crate::source::Source;

pub(crate) struct QueueSource {
    pub(crate) source: Box<dyn Source>,
    pub(crate) enable_resampling: bool,
    pub(crate) force_restart_output: bool,
}
