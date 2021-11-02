mod management;
mod player;
mod rpc;
mod server;
mod signal_handler;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use crate::signal_handler::platform::SignalHandler;
#[cfg(unix)]
use crate::unix::unix_stream::UnixStream;
use anyhow::Result;
use rpc::*;
use std::panic;
use tokio::sync::broadcast;
use tracing::error;
use tracing::info;
#[cfg(windows)]
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
#[cfg(windows)]
use windows::service;

fn set_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            error!("panic occurred: {:?}", s);
        } else {
            error!("panic occurred: {:?}", panic_info);
        }
    }));
}

#[tokio::main]
async fn main() {
    let proj_dirs = directories::ProjectDirs::from("", "", "platune")
        .expect("Unable to find a valid home directory");
    let file_appender = tracing_appender::rolling::hourly(proj_dirs.cache_dir(), "platuned.log");
    let (non_blocking_stdout, _stdout_guard) = tracing_appender::non_blocking(std::io::stdout());
    let (non_blocking_file, _file_guard) = tracing_appender::non_blocking(file_appender);

    let collector = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with({
            #[allow(clippy::let_and_return)]
            let layer = Layer::new()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_ansi(false)
                .with_writer(non_blocking_file);

            #[cfg(windows)]
            let layer = layer.with_timer(LocalTime::rfc_3339());

            layer
        })
        .with({
            #[allow(clippy::let_and_return)]
            let layer = Layer::new()
                .pretty()
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_writer(non_blocking_stdout);

            #[cfg(windows)]
            let layer = layer.with_timer(LocalTime::rfc_3339());

            layer
        });

    // This has to be ran directly in the main function
    tracing::subscriber::set_global_default(collector)
        .expect("Unable to set global tracing subscriber");

    // Don't set panic hook until after logging is set up
    set_panic_hook();

    info!("starting");
    if let Err(e) = os_main().await {
        error!("{:?}", e);

        std::process::exit(1);
    }
}

#[cfg(windows)]
async fn os_main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "-i" {
        service::install()?;
    } else if args.len() > 1 && args[1] == "-s" {
        dotenv::from_path(r"C:\Users\asche\code\platune\platuned\server\.env").unwrap();
        service::run()?;
    } else {
        let (tx, _) = broadcast::channel(32);
        dotenv::from_path("./.env").unwrap();
        let signal_handler = SignalHandler::start(tx.clone())?;
        server::run_all(tx).await?;
        signal_handler.close().await?;
    }

    Ok(())
}

#[cfg(unix)]
async fn os_main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if !(args.len() > 1 && args[1] == "-s") {
        dotenv::from_path("./.env").unwrap();
    }

    let (tx, _) = broadcast::channel(32);
    let signal_handler = SignalHandler::start(tx.clone())?;
    server::run_all(tx).await?;
    signal_handler.close().await
}
