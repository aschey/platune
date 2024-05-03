use std::num::NonZeroUsize;
use std::{env, fs};

use decal::decoder::{ReadSeekSource, Source};
use eyre::{Context, Result};
use stream_download::http::reqwest::{Client, Identity};
use stream_download::http::HttpStream;
use stream_download::source::SourceStream;
use stream_download::storage::adaptive::AdaptiveStorageProvider;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use tracing::info;

#[derive(Debug)]
pub(crate) struct HttpStreamReader {
    downloader: StreamDownload<AdaptiveStorageProvider<TempStorageProvider>>,
    url: String,
    file_len: Option<u64>,
}

impl HttpStreamReader {
    pub async fn new(url: String) -> Result<Self> {
        let mut client_builder = Client::builder();
        if url.starts_with("https://") {
            let mtls_cert_path = env::var("PLATUNE_MTLS_CLIENT_CERT_PATH");
            let mtls_key_path = env::var("PLATUNE_MTLS_CLIENT_KEY_PATH");
            if let (Ok(mtls_cert_path), Ok(mtls_key_path)) = (mtls_cert_path, mtls_key_path) {
                info!("Using cert paths: {mtls_cert_path} {mtls_key_path}");
                let mut cert =
                    fs::read(mtls_cert_path).wrap_err_with(|| "mtls cert path invalid")?;
                let mut key = fs::read(mtls_key_path).wrap_err_with(|| "mtls key path invalid")?;
                cert.append(&mut key);

                client_builder = client_builder.identity(Identity::from_pem(&cert)?);
            }
        }
        let stream = HttpStream::new(client_builder.build()?, url.parse()?)
            .await
            .wrap_err_with(|| "Error creating http stream")?;
        let file_len = stream.content_length();
        let settings = Settings::default();
        Ok(Self {
            url: url.clone(),
            downloader: StreamDownload::from_stream(
                stream,
                // store 512 kb of audio when the content length is not known
                AdaptiveStorageProvider::new(
                    TempStorageProvider::default(),
                    NonZeroUsize::new(1024 * 512).expect("nonzero"),
                ),
                settings,
            )
            .await
            .wrap_err_with(|| "Error creating stream downloader")?,
            file_len,
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

        Box::new(ReadSeekSource::new(
            self.downloader,
            self.file_len,
            extension,
        ))
    }
}
