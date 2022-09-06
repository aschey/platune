use crate::server;
use daemon_slayer_server::{HandlerAsync, StopHandlerAsync};
use tokio::sync::broadcast;
use tracing::info;

#[derive(daemon_slayer::server::ServiceAsync)]
pub struct ServiceHandler {
    tx: broadcast::Sender<()>,
}

#[daemon_slayer::server::async_trait::async_trait]
impl HandlerAsync for ServiceHandler {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(32);
        Self { tx }
    }

    fn get_service_name<'a>() -> &'a str {
        "platuned"
    }

    fn get_stop_handler(&mut self) -> StopHandlerAsync {
        let tx = self.tx.clone();
        Box::new(move || {
            let tx = tx.clone();
            Box::pin(async move {
                info!("stopping");
                tx.send(()).unwrap();
            })
        })
    }

    async fn run_service<F: FnOnce() + Send>(mut self, on_started: F) -> u32 {
        info!("running service");
        on_started();
        server::run_all(self.tx).await.unwrap();
        0
    }
}
