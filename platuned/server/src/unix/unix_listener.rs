use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use tracing::error;

pub struct UnixListener {
    path: PathBuf,
    listener: tokio::net::UnixListener,
}

impl UnixListener {
    pub fn bind(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_owned();
        if let Err(e) = std::fs::remove_file(&path) {
            if e.kind() != ErrorKind::NotFound {
                return Err(e).with_context(|| "Unable to delete old Unix socket");
            }
        }
        Ok(
            tokio::net::UnixListener::bind(&path)
                .map(|listener| UnixListener { path, listener })?,
        )
    }
}

impl Drop for UnixListener {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.path) {
            if e.kind() != ErrorKind::NotFound {
                error!("Unable to delete old Unix socket: {:?}", e);
            }
        }
    }
}

impl std::ops::Deref for UnixListener {
    type Target = tokio::net::UnixListener;

    fn deref(&self) -> &Self::Target {
        &self.listener
    }
}

impl std::ops::DerefMut for UnixListener {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.listener
    }
}
