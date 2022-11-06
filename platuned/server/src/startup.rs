use std::{env::var, error::Error, sync::Arc};

use crate::{
    rpc::*, server, sync_handler::SyncHandler, sync_handler_builder::SyncHandlerBuilder,
    sync_handler_client::SyncHandlerClient, sync_processor::SyncProcessor,
};
use daemon_slayer::{
    error_handler::color_eyre::eyre::{self, Context},
    server::{BroadcastEventStore, Handler, ServiceContext, SubsystemHandle},
    signals::{Signal, SignalHandler, SignalHandlerBuilder, SignalHandlerBuilderTrait},
    task_queue::{TaskQueue, TaskQueueBuilder},
};
use libplatune_management::{
    config::FileConfig,
    database::Database,
    file_watcher::{
        file_watch_builder::FileWatchBuilder, file_watch_manager::FileWatchManager,
        file_watch_service::FileWatchService,
    },
    manager::Manager,
};
use tracing::info;

#[derive(daemon_slayer::server::Service)]
pub struct ServiceHandler {
    subsys: SubsystemHandle,
    progress_store: BroadcastEventStore<Progress>,
    manager: FileWatchManager,
    sync_client: SyncHandlerClient,
}

#[async_trait::async_trait]
impl Handler for ServiceHandler {
    async fn new(context: &mut ServiceContext) -> Self {
        let subsys = context.get_subsystem_handle();
        context
            .add_event_service::<SignalHandler>(SignalHandlerBuilder::all())
            .await;
        let manager = init_manager().await.unwrap();
        let (path_tx, path_rx) = tokio::sync::mpsc::channel(32);
        let file_watch_manager = context
            .add_service::<FileWatchService>(FileWatchBuilder::new(manager, path_tx.clone()))
            .await;

        let sync_processor = SyncProcessor::new(file_watch_manager.clone());
        let progress_store = sync_processor.get_event_store();

        let (task_queue_client, _) = context
            .add_event_service::<TaskQueue>(
                TaskQueueBuilder::default().with_job_handler(sync_processor),
            )
            .await;
        let sync_client = context
            .add_service::<SyncHandler>(SyncHandlerBuilder::new(
                file_watch_manager.clone(),
                task_queue_client,
                path_tx,
                path_rx,
            ))
            .await;

        Self {
            subsys,
            manager: file_watch_manager,
            sync_client,
            progress_store,
        }
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

        server::run_all(
            self.manager,
            self.sync_client,
            self.progress_store,
            self.subsys,
        )
        .await?;
        Ok(())
    }
}

async fn init_manager() -> eyre::Result<Manager> {
    let path = var("DATABASE_URL").wrap_err("DATABASE_URL environment variable not set")?;
    let db = Database::connect(path, true).await?;
    db.migrate().await.wrap_err("Error migrating database")?;
    let config = Arc::new(FileConfig::try_new()?);
    let manager = Manager::new(&db, config);

    Ok(manager)
}
