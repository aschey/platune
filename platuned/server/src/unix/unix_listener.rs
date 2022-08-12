use color_eyre::eyre::{self, Context, Result};
use eyre::eyre;
use std::{
    io::ErrorKind,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use tracing::error;

pub struct UnixListener {
    path: PathBuf,
    listener: tokio::net::UnixListener,
}

impl UnixListener {
    pub fn bind(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_owned();

        Self::create_socket_path(&path)?;

        let listener =
            tokio::net::UnixListener::bind(&path).wrap_err("Error binding to Unix socket")?;

        let mut perms = path
            .metadata()
            .wrap_err("Error reading metadata from Unix socket file")?
            .permissions();
        perms.set_mode(0o666);
        std::fs::set_permissions(&path, perms)
            .wrap_err("Error setting permissions on Unix socket file")?;

        Ok(Self { path, listener })
    }

    fn create_socket_path(path: &Path) -> Result<()> {
        let parent_dir = path
            .parent()
            .ok_or_else(|| eyre!("Socket path should have a parent directory"))?;
        if let Err(e) = std::fs::remove_file(path) {
            if e.kind() != ErrorKind::NotFound {
                return Err(e).wrap_err("Unable to delete old Unix socket");
            }
        }

        std::fs::create_dir_all(parent_dir).wrap_err("Unable to create Unix socket directory")?;
        let mut perms = parent_dir
            .metadata()
            .wrap_err("Error setting socket directory metadata")?
            .permissions();
        perms.set_mode(0o644);
        Ok(())
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
