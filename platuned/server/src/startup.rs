use std::error::Error;

use daemon_slayer::core::{BoxedError, Label};
use daemon_slayer::server::{
    BroadcastEventStore, Handler, Service, ServiceContext, Signal, SignalHandler,
};
use daemon_slayer::signals::SignalListener;
use platuned::service_label;
use tracing::info;

use crate::server;

#[derive(Service)]
pub struct ServiceHandler {
    signal_store: BroadcastEventStore<Signal>,
}

impl Handler for ServiceHandler {
    type Error = BoxedError;
    type InputData = ();

    async fn new(context: ServiceContext, _: Option<Self::InputData>) -> Result<Self, Self::Error> {
        let signal_listener = SignalListener::termination();
        let signal_store = signal_listener.get_event_store();
        context.spawn(signal_listener);

        Ok(Self { signal_store })
    }

    fn label() -> Label {
        service_label()
    }

    async fn run_service<F: FnOnce() + Send>(
        self,
        notify_ready: F,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("running service");
        notify_ready();

        server::run_all(self.signal_store.clone()).await?;
        Ok(())
    }
}
