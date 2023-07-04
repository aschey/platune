use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct CurrentPosition {
    pub position: Duration,
    pub retrieval_time: Option<Duration>,
}
