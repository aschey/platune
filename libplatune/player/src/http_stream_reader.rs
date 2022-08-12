use eyre::{eyre, Context, Result};
use flume::{Receiver, Sender};
use futures_util::StreamExt;
use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    str::FromStr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use tap::TapFallible;
use tempfile::{Builder, NamedTempFile};
use tracing::{error, info};

use crate::source::{ReadSeekSource, Source};

#[derive(Debug)]
pub(crate) struct HttpStreamReader {
    output_reader: BufReader<NamedTempFile>,
    bytes_written: Arc<AtomicU32>,
    bytes_read: u32,
    wait_written_tx: Sender<u32>,
    ready_rx: Receiver<u64>,
    url: String,
    pub file_len: u64,
}

impl HttpStreamReader {
    pub async fn new(url: String) -> Result<Self> {
        let bytes_written = Arc::new(AtomicU32::new(0));
        let bytes_written_ = bytes_written.clone();

        let (ready_tx, ready_rx) = flume::unbounded();
        let (wait_written_tx, wait_written_rx) = flume::unbounded();

        let tempfile_ = Builder::new()
            .prefix("platunecache")
            .tempfile()
            .wrap_err("Error creating http stream reader temp file")?;

        let tempfile = tempfile_
            .reopen()
            .wrap_err("Error reading http stream reader temp file")?;
        let url_ = url.clone();

        tokio::spawn(async move {
            let _ = Self::download_file(&url_, tempfile, bytes_written_, wait_written_rx, ready_tx)
                .await
                .tap_err(|e| error!("Error downloading file {:?}", e));
        });

        let file_len = ready_rx
            .recv_async()
            .await
            .wrap_err("Error receiving ready signal from file http file download")?;

        let output_reader = BufReader::new(tempfile_);
        Ok(HttpStreamReader {
            output_reader,
            bytes_read: 0,
            bytes_written,
            wait_written_tx,
            ready_rx,
            file_len,
            url,
        })
    }

    pub fn into_source(self) -> Box<dyn Source> {
        let parts: Vec<&str> = self.url.split('.').collect();
        let extension = if parts.len() > 1 {
            parts.last().map(|e| e.to_string())
        } else {
            None
        };
        info!("Using extension {extension:?}");

        let file_len = self.file_len;
        let reader = BufReader::new(self);

        Box::new(ReadSeekSource::new(reader, Some(file_len), extension))
    }

    async fn download_file(
        url: &str,
        mut tempfile: File,
        bytes_written: Arc<AtomicU32>,
        wait_written_rx: Receiver<u32>,
        ready_tx: Sender<u64>,
    ) -> Result<()> {
        info!("Starting download...");
        let client = reqwest::Client::new();
        let res = client
            .head(url)
            .send()
            .await
            .wrap_err(format!("Error sending HEAD request to url {}", url))?;

        let file_len = res
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .ok_or_else(|| eyre!("Content length missing"))?;
        let file_len = u64::from_str(file_len.to_str().wrap_err("Invalid content length")?)
            .wrap_err("Invalid content length")?;

        info!("File length={}", file_len);

        let mut stream = client
            .get(url)
            .send()
            .await
            .wrap_err(format!("Error fetching url {}", url))?
            .bytes_stream();

        // Pre-buffer 1% of the file
        let mut target = (file_len as f32 * 0.01) as u32;
        while let Some(item) = stream.next().await {
            let item = item.wrap_err("Error receiving bytes from http download")?;
            let len = item.len();
            std::io::copy(&mut item.take(len as u64), &mut tempfile)
                .wrap_err("Error copying http download data to temp file")?;
            let new_len = bytes_written.load(Ordering::SeqCst) + len as u32;
            bytes_written.store(new_len, Ordering::SeqCst);

            if let Ok(new_target) = wait_written_rx.try_recv() {
                info!("write target={}", new_target);
                target = new_target;
            }
            if new_len >= target {
                info!("Reached target");
                ready_tx
                    .send_async(file_len)
                    .await
                    .wrap_err("Error sending http file download ready signal")?;

                // reset target to prevent this branch from triggering again
                target = u32::MAX;
            }
        }

        info!("Finished downloading file");
        Ok(())
    }

    fn wait_for_download(&mut self, requested_len: u32) {
        let written = self.bytes_written.load(Ordering::SeqCst);

        if written < requested_len {
            self.wait_written_tx.send(requested_len).unwrap_or_default();
            self.ready_rx.recv().unwrap_or_default();
            info!("Finished waiting for write target");
        }
    }
}

impl Read for HttpStreamReader {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let requested_len = self.bytes_read + buf.len() as u32;
        self.wait_for_download(requested_len);

        let res = self.output_reader.read(buf);
        self.bytes_read += buf.len() as u32;
        res
    }
}

impl Seek for HttpStreamReader {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, std::io::Error> {
        let requested_len = match pos {
            SeekFrom::Start(time) => time as i64,
            SeekFrom::End(time) => self.file_len as i64 - time,
            SeekFrom::Current(time) => self.bytes_read as i64 + time,
        } as u32;
        self.wait_for_download(requested_len);
        self.output_reader.seek(pos)
    }
}
