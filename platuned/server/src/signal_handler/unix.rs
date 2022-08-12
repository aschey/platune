use color_eyre::eyre::Result;
use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::{Handle, Signals};
use tokio::sync::broadcast::Sender;
use tokio::task::JoinHandle;
use tracing::info;
use tracing::log::error;

pub struct SignalHandler {
    handle: Handle,
    signals_task: JoinHandle<()>,
}

impl SignalHandler {
    async fn handle_signals(signals: Signals, tx: Sender<()>) {
        let mut signals = signals.fuse();
        while let Some(signal) = signals.next().await {
            info!("Received signal: {:?}", signal);
            match signal {
                SIGTERM | SIGINT | SIGQUIT | SIGHUP => {
                    info!("Sending shutdown signal");
                    if let Err(e) = tx.send(()) {
                        error!("Error sending shutdown signal {:?}", e);
                    }
                }
                _ => {
                    info!("Ignoring signal");
                }
            }
        }
    }

    pub fn start(tx: Sender<()>) -> Result<Self> {
        let signals = Signals::new(&[SIGHUP, SIGTERM, SIGINT, SIGQUIT])?;
        let handle = signals.handle();

        let signals_task = tokio::spawn(Self::handle_signals(signals, tx));

        Ok(Self {
            handle,
            signals_task,
        })
    }

    pub async fn close(self) -> Result<()> {
        self.handle.close();
        Ok(self.signals_task.await?)
    }
}
