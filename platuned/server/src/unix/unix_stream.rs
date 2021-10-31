use anyhow::Context;
use async_stream::AsyncStream;
use futures::TryFutureExt;
use std::{
    io::{ErrorKind, Result},
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{Context as TaskContext, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::unix::{SocketAddr, UCred},
};
use tonic::transport::server::Connected;
use tracing::error;

use crate::unix::unix_listener::UnixListener;

#[derive(Debug)]
pub struct UnixStream(pub tokio::net::UnixStream);

impl UnixStream {
    pub fn get_async_stream(
        path: impl AsRef<Path>,
        fallback_to_tmp: bool,
    ) -> anyhow::Result<
        AsyncStream<anyhow::Result<UnixStream, std::io::Error>, impl futures::Future<Output = ()>>,
    > {
        let mut path = path.as_ref();

        if let Err(e) = Self::create_socket_path(path) {
            if fallback_to_tmp {
                path = Path::new("/tmp/platuned");
                Self::create_socket_path(path)?;
            } else {
                return Err(e);
            }
        }

        {
            let uds = UnixListener::bind(path).with_context(|| "Error binding to Unix socket")?;

            Ok(async_stream::stream! {
                loop {
                    let item = uds.accept().map_ok(|(st, _)| UnixStream(st)).await;

                    yield item;
                }
            })
        }
    }

    fn create_socket_path(path: &Path) -> anyhow::Result<()> {
        let parent_dir = path
            .parent()
            .with_context(|| "Socket path should have a parent directory")?;
        if let Err(e) = std::fs::remove_file(path) {
            if e.kind() != ErrorKind::NotFound {
                error!("Unable to delete old Unix socket: {:?}", e);
            }
        }

        std::fs::create_dir_all(parent_dir)
            .with_context(|| "Unable to create Unix socket directory")
    }
}

impl Connected for UnixStream {
    type ConnectInfo = UdsConnectInfo;

    fn connect_info(&self) -> Self::ConnectInfo {
        UdsConnectInfo {
            peer_addr: self.0.peer_addr().ok().map(Arc::new),
            peer_cred: self.0.peer_cred().ok(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UdsConnectInfo {
    pub peer_addr: Option<Arc<SocketAddr>>,
    pub peer_cred: Option<UCred>,
}

impl AsyncRead for UnixStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for UnixStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
