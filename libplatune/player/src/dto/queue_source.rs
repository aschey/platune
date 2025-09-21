use decal::decoder::Source;

use super::track::Metadata;
use crate::settings::Settings;

#[derive(Debug)]
pub(crate) struct QueueSource {
    pub(crate) source: Box<dyn Source>,
    pub(crate) metadata: Metadata,
    pub(crate) volume: Option<f32>,
    pub(crate) settings: Settings,
}
