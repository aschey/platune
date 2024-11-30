use std::io::BufReader;
use std::num::NonZeroUsize;
use std::path::Path;
use std::{env, fs};

use async_trait::async_trait;
use decal::decoder::{ReadSeekSource, Source};
use eyre::{Context, Result, eyre};
use reqwest_retry::RetryTransientMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use stream_download::http::HttpStream;
use stream_download::http::reqwest::{Client, Identity, Url};
use stream_download::registry::{Input, RegistryEntry, Rule};
use stream_download::source::{DecodeError, SourceStream};
use stream_download::storage::adaptive::AdaptiveStorageProvider;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use tap::{TapFallible, TapOptional};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

fn file_rule() -> Rule {
    Rule::url_scheme("file://")
}

pub(crate) struct DefaultUrlResolver {
    rules: Vec<Rule>,
}

impl DefaultUrlResolver {
    pub(crate) fn new() -> Self {
        Self {
            rules: vec![Rule::any_http(), file_rule(), Rule::any_string()],
        }
    }
}

#[async_trait]
impl RegistryEntry<Result<Vec<Input>>> for DefaultUrlResolver {
    fn priority(&self) -> u32 {
        2
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    async fn handler(&mut self, input: Input) -> Result<Vec<Input>> {
        Ok(vec![input])
    }
}

pub(crate) struct HttpSourceResolver {
    rules: Vec<Rule>,
}

impl HttpSourceResolver {
    pub(crate) fn new() -> Self {
        Self {
            rules: vec![Rule::any_http()],
        }
    }
}

#[async_trait::async_trait]
impl RegistryEntry<Result<(Box<dyn Source>, CancellationToken)>> for HttpSourceResolver {
    fn priority(&self) -> u32 {
        2
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    async fn handler(&mut self, input: Input) -> Result<(Box<dyn Source>, CancellationToken)> {
        let mut client_builder = Client::builder();
        let url = input.source.into_url();
        if url.scheme() == "https" {
            if let Ok(platune_server_url) = env::var("PLATUNE_GLOBAL_FILE_URL") {
                let platune_server_url: Url = platune_server_url.parse()?;
                if url.host_str() == platune_server_url.host_str() {
                    let mtls_cert_path = env::var("PLATUNE_MTLS_CLIENT_CERT_PATH");
                    let mtls_key_path = env::var("PLATUNE_MTLS_CLIENT_KEY_PATH");
                    if let (Ok(mtls_cert_path), Ok(mtls_key_path)) = (mtls_cert_path, mtls_key_path)
                    {
                        info!("Using cert paths: {mtls_cert_path} {mtls_key_path}");
                        let mut cert =
                            fs::read(mtls_cert_path).wrap_err_with(|| "mtls cert path invalid")?;
                        let mut key =
                            fs::read(mtls_key_path).wrap_err_with(|| "mtls key path invalid")?;
                        cert.append(&mut key);

                        client_builder = client_builder.identity(Identity::from_pem(&cert)?);
                    }
                }
            }
        }
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = reqwest_middleware::ClientBuilder::new(client_builder.build()?)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        let stream = match HttpStream::new(client, url.clone()).await {
            Ok(stream) => stream,
            Err(e) => {
                return Err(eyre!(
                    "Error creating http stream: {}",
                    e.decode_error().await
                ));
            }
        };

        let file_len = stream.content_length();

        let parts: Vec<&str> = url.path().split('.').collect();
        let extension = if parts.len() > 1 {
            parts.last().map(|e| e.to_string())
        } else {
            None
        };
        info!("Using extension {extension:?}");
        let settings = Settings::default();
        // radio streams commonly include an Icy-Br header to denote the bitrate
        let prefetch_bytes =
            if let Some(Ok(bitrate)) = stream.header("Icy-Br").map(|br| br.parse::<u64>()) {
                // buffer 5 seconds of audio
                // bitrate (in kilobits) / bits per byte * bytes per kilobyte * 5 seconds
                bitrate / 8 * 1024 * 5
            } else {
                settings.get_prefetch_bytes()
            };

        let reader = StreamDownload::from_stream(
            stream,
            // store 512 kb of audio when the content length is not known
            AdaptiveStorageProvider::new(
                TempStorageProvider::with_prefix("platune_cache"),
                NonZeroUsize::new(1024 * 512).expect("nonzero"),
            ),
            settings.prefetch_bytes(prefetch_bytes),
        )
        .await
        .wrap_err_with(|| "Error creating stream downloader")?;
        let token = reader.get_cancellation_token();

        Ok((
            Box::new(ReadSeekSource::new(reader, file_len, extension)),
            token,
        ))
    }
}

pub(crate) struct FileSourceResolver {
    rules: Vec<Rule>,
}

impl FileSourceResolver {
    pub(crate) fn new() -> Self {
        Self {
            rules: vec![file_rule(), Rule::any_string()],
        }
    }
}

#[async_trait::async_trait]
impl RegistryEntry<Result<(Box<dyn Source>, CancellationToken)>> for FileSourceResolver {
    fn priority(&self) -> u32 {
        3
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    async fn handler(&mut self, input: Input) -> Result<(Box<dyn Source>, CancellationToken)> {
        let path = input.source.to_string();
        let file = fs::File::open(&path).tap_err(|e| error!("Error opening file {path} {e:?}"))?;

        let file_len = file
            .metadata()
            .map(|m| m.len())
            .tap_err(|e| warn!("Error reading file metadata from {path}: {e:?}"))
            .ok();

        let extension = Path::new(&path)
            .extension()
            .and_then(|ext| ext.to_str())
            .tap_none(|| {
                warn!(
                    "File extension for {path} contains invalid unicode. Not using extension hint"
                )
            })
            .map(|ext| ext.to_owned());

        let reader = BufReader::new(file);

        Ok((
            Box::new(ReadSeekSource::new(reader, file_len, extension)),
            CancellationToken::new(),
        ))
    }
}
