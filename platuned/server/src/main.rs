mod rpc;
mod server;
mod services;
mod signal_handler;
mod startup;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use rpc::*;
use std::panic;
use tracing::error;
use tracing::info;
#[cfg(windows)]
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

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
    let (non_blocking_stdout, stdout_guard) = tracing_appender::non_blocking(std::io::stdout());
    let (non_blocking_file, file_guard) = tracing_appender::non_blocking(file_appender);

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
    if let Err(e) = startup::start().await {
        error!("{:?}", e.to_string());

        // Drop guards to ensure logs are flushed
        drop(stdout_guard);
        drop(file_guard);
        std::process::exit(1);
    }
}
