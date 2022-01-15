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
use tracing::error;
use tracing::info;
#[cfg(windows)]
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

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

#[tokio::main]
async fn main() {
    let proj_dirs =
        ProjectDirs::from("", "", "platune").expect("Unable to find a valid home directory");
    let log_dir = proj_dirs.cache_dir();
    let file_appender = tracing_appender::rolling::hourly(log_dir, "platuned.log");

    // The default number of buffered lines is quite large and uses a ton of memory
    // We aren't logging a ton of messages so setting this value somewhat low is fine in order to conserve memory
    let (non_blocking_stdout, stdout_guard) =
        tracing_appender::non_blocking::NonBlockingBuilder::default()
            .buffered_lines_limit(256)
            .finish(stdout());

    let (non_blocking_file, file_guard) =
        tracing_appender::non_blocking::NonBlockingBuilder::default()
            .buffered_lines_limit(256)
            .finish(file_appender);

    let collector = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(console_subscriber::spawn())
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
    info!("Log dir: {:?}", log_dir);
    info!("Starting...");
    if let Err(e) = startup::start().await {
        error!("{:?}", e);

        // Drop guards to ensure logs are flushed
        drop(stdout_guard);
        drop(file_guard);
        exit(1);
    }
}
