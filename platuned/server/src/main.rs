mod rpc;
mod server;
mod services;
mod signal_handler;
mod startup;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use directories::ProjectDirs;
use rpc::*;
use std::io::stdout;
use std::panic;
use std::process::exit;
use time::format_description::well_known;
use time::UtcOffset;
use tracing::error;
use tracing::info;
use tracing::log::warn;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::{
    filter::LevelFilter, fmt::Layer, layer::SubscriberExt, EnvFilter, Layer as SubscriberLayer,
};

fn set_panic_hook() {
    panic::set_hook(Box::new(|panic_info| {
        if let Some(location) = panic_info.location() {
            error!(
                message = %panic_info,
                panic.file = location.file(),
                panic.line = location.line(),
                panic.column = location.column(),
            );
        } else {
            error!(message = %panic_info);
        }
    }));
}

fn main() {
    // IMPORTANT: retrieving the timezone must be done before the program spawns any threads,
    // which means it must be done before the Tokio runtime is initialized
    // To be safe, it should be the first line in the program
    let (offset, is_local) = match OffsetTime::local_rfc_3339() {
        Ok(offset) => (offset, true),
        Err(_) => (OffsetTime::new(UtcOffset::UTC, well_known::Rfc3339), false),
    };

    let proj_dirs =
        ProjectDirs::from("", "", "platune").expect("Unable to find a valid home directory");
    let log_dir = proj_dirs.cache_dir();
    let file_appender = tracing_appender::rolling::hourly(log_dir, "platuned.log");

    // The default number of buffered lines is quite large and uses a ton of memory
    // We aren't logging a ton of messages so setting this value somewhat low is fine in order to conserve memory
    let buffer_limit = 256;
    let (non_blocking_stdout, stdout_guard) =
        tracing_appender::non_blocking::NonBlockingBuilder::default()
            .buffered_lines_limit(buffer_limit)
            .finish(stdout());

    let (non_blocking_file, file_guard) =
        tracing_appender::non_blocking::NonBlockingBuilder::default()
            .buffered_lines_limit(buffer_limit)
            .finish(file_appender);

    let level = if cfg!(feature = "console") {
        tracing::Level::TRACE
    } else {
        tracing::Level::INFO
    };

    let collector = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(level.into()))
        .with({
            #[allow(clippy::let_and_return)]
            let layer = Layer::new()
                .with_timer(offset.clone())
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_ansi(false)
                .with_writer(non_blocking_file);

            layer.with_filter(LevelFilter::INFO)
        })
        .with({
            #[allow(clippy::let_and_return)]
            let layer = Layer::new()
                .pretty()
                .with_timer(offset)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_writer(non_blocking_stdout);

            layer.with_filter(LevelFilter::INFO)
        });

    #[cfg(feature = "console")]
    let collector = collector.with(console_subscriber::spawn());

    // This has to be ran directly in the main function
    tracing::subscriber::set_global_default(collector)
        .expect("Unable to set global tracing subscriber");

    // Don't set panic hook until after logging is set up
    set_panic_hook();

    if !is_local {
        warn!("Using UTC time for logging because the local offset wasn't determined");
    }
    info!("Log dir: {:?}", log_dir);
    info!("Starting...");

    run_async(file_guard, stdout_guard);
}

#[tokio::main]
async fn run_async(file_guard: WorkerGuard, stdout_guard: WorkerGuard) {
    if let Err(e) = startup::start().await {
        error!("{:?}", e);

        // Drop guards to ensure logs are flushed
        drop(stdout_guard);
        drop(file_guard);
        exit(1);
    }
}
