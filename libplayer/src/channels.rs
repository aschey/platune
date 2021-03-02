#[cfg(feature = "runtime-tokio")]
pub mod mpsc {
    pub use tokio::sync::mpsc::*;
    pub fn async_channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        channel(capacity)
    }
    pub async fn recv<T>(rx: &mut Receiver<T>) -> Result<T, &'static str> {
        rx.recv().await.ok_or_else(|| "Channel closed")
    }
}

#[cfg(feature = "runtime-async-std")]
pub mod mpsc {
    pub use async_std::channel::*;
    pub fn async_channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
        bounded(capacity)
    }
    pub async fn recv<T>(rx: &mut Receiver<T>) -> Result<T, RecvError> {
        rx.recv().await
    }
}
