use anyhow::Result;
use tokio::sync::broadcast::Sender;
use tracing::{info, log::error};

pub struct SignalHandler;

impl SignalHandler {
    pub fn start(tx: Sender<()>) -> Result<Self> {
        ctrlc::set_handler(move || {
            info!("Sending shutdown signal");
            if let Err(e) = tx.send(()) {
                error!("Error sending shutdown signal {:?}", e);
            }
        })?;

        Ok(Self)
    }

    pub async fn close(self) -> Result<()> {
        Ok(())
    }
}
