use std::io::{self};
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};

use futures::{Stream, StreamExt};
use tipsy::{Endpoint, IntoIpcPath, OnConflict};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tonic::transport::server::Connected;

trait ReadWrite: AsyncRead + AsyncWrite {}

impl<T> ReadWrite for T where T: AsyncRead + AsyncWrite {}

pub struct IpcStream(Pin<Box<dyn ReadWrite + Send>>);

impl IpcStream {
    pub fn get_async_stream(
        path: impl IntoIpcPath,
    ) -> io::Result<impl Stream<Item = io::Result<IpcStream>>> {
        let stream = Endpoint::new(path, OnConflict::Overwrite)?.incoming()?;
        Ok(stream.map(|next| next.map(|s| IpcStream(Box::pin(s)))))
    }
}

impl Connected for IpcStream {
    type ConnectInfo = ();

    fn connect_info(&self) -> Self::ConnectInfo {}
}

impl AsyncRead for IpcStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl AsyncWrite for IpcStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
