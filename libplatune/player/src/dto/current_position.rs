use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CurrentPosition {
    pub position: Duration,
    pub retrieval_time: Duration,
}
