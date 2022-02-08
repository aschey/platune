use std::task::Poll;

use futures::StreamExt;
use tokio::sync::broadcast;
use tokio_stream::wrappers::{errors::BroadcastStreamRecvError, BroadcastStream};

use super::sync_engine::SyncError;

pub struct ProgressStream {
    inner: BroadcastStream<Option<Result<f32, SyncError>>>,
    last_val: Option<Result<f32, SyncError>>,
}

impl ProgressStream {
    pub fn new(rx: broadcast::Receiver<Option<Result<f32, SyncError>>>) -> Self {
        Self {
            inner: BroadcastStream::new(rx),
            last_val: Some(Ok(0.0)),
        }
    }
}

impl futures::Stream for ProgressStream {
    type Item = Result<f32, SyncError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.inner.poll_next_unpin(cx) {
            Poll::Ready(progress_val_option) => match progress_val_option {
                None => Poll::Ready(None),
                Some(progress_val_result) => match progress_val_result {
                    Ok(progress_val) => {
                        self.last_val = progress_val.clone();
                        Poll::Ready(progress_val)
                    }
                    Err(BroadcastStreamRecvError::Lagged(_)) => Poll::Ready(self.last_val.clone()),
                },
            },
            Poll::Pending => Poll::Pending,
        }
    }
}
