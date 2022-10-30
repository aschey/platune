use std::error::Error;

use crate::server;
use daemon_slayer::{
    server::{BroadcastEventStore, Handler, ServiceContext},
    signals::{Signal, SignalHandler, SignalHandlerBuilder, SignalHandlerBuilderTrait},
};
use tracing::info;

#[derive(daemon_slayer::server::Service)]
pub struct ServiceHandler {
    signal_store: BroadcastEventStore<Signal>,
}

#[async_trait::async_trait]
impl Handler for ServiceHandler {
    async fn new(context: &mut ServiceContext) -> Self {
        let (_, signal_store) = context
            .add_event_service::<SignalHandler>(SignalHandlerBuilder::all())
            .await;

        Self { signal_store }
    }

    fn get_service_name<'a>() -> &'a str {
        "platuned"
    }

    async fn run_service<F: FnOnce() + Send>(
        mut self,
        on_started: F,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("running service");
        on_started();

        server::run_all(self.signal_store.clone()).await?;
        Ok(())
    }
}
