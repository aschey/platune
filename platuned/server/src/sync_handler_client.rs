use daemon_slayer::error_handler::color_eyre::eyre;

#[derive(Clone)]
pub struct SyncHandlerClient {
    tx: tokio::sync::mpsc::Sender<Option<Vec<String>>>,
}

impl SyncHandlerClient {
    pub fn new(tx: tokio::sync::mpsc::Sender<Option<Vec<String>>>) -> Self {
        Self { tx }
    }
    pub async fn start_sync(&self) -> eyre::Result<()> {
        self.tx.send(None).await?;
        Ok(())
    }
}
