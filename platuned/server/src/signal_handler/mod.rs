#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub mod platform {
    pub use super::unix::SignalHandler;
}

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub mod platform {
    pub use super::windows::SignalHandler;
}
