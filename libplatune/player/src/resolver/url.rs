use std::io::BufReader;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};

use async_trait::async_trait;
use decal::decoder::ReadSeekSource;
use eyre::{Context, Result, eyre};
use icy_metadata::{IcyHeaders, IcyMetadataReader, RequestIcyMetadata};
use reqwest_retry::RetryTransientMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use stream_download::http::HttpStream;
use stream_download::http::reqwest::{Client, Identity, Url};
use stream_download::registry::{self, Input, RegistryEntry, Rule};
use stream_download::source::{DecodeError, SourceStream};
use stream_download::storage::adaptive::AdaptiveStorageProvider;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use tap::{TapFallible, TapOptional};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use super::MetadataSource;
use crate::dto::track::Metadata;

pub(crate) struct DefaultUrlResolver {
    rules: Vec<Rule>,
}

impl DefaultUrlResolver {
    pub(crate) fn new() -> Self {
        Self {
            rules: vec![Rule::any_url(), Rule::any_string()],
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
        if let registry::Source::Url(url) = &input.source
            && let Ok(path) = url.to_file_path()
        {
            return Ok(vec![Input {
                prefix: None,
                source: registry::Source::String(path.to_string_lossy().to_string()),
            }]);
        }

        Ok(vec![input])
    }
}

pub(crate) struct HttpSourceResolver {
    rules: Vec<Rule>,
    on_track_changed: Arc<dyn Fn(Metadata) + Send + Sync>,
}

impl HttpSourceResolver {
    pub(crate) fn new(on_track_changed: Arc<dyn Fn(Metadata) + Send + Sync>) -> Self {
        Self {
            rules: vec![Rule::any_http()],
            on_track_changed,
        }
    }
}

#[async_trait::async_trait]
impl RegistryEntry<Result<(MetadataSource, CancellationToken)>> for HttpSourceResolver {
    fn priority(&self) -> u32 {
        2
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    async fn handler(&mut self, input: Input) -> Result<(MetadataSource, CancellationToken)> {
        let mut client_builder = Client::builder();
        let url = input.source.into_url();
        if url.scheme() == "https"
            && let Ok(platune_server_url) = env::var("PLATUNE_GLOBAL_FILE_URL")
        {
            let platune_server_url: Url = platune_server_url.parse()?;
            if url.host_str() == platune_server_url.host_str() {
                let mtls_cert_path = env::var("PLATUNE_MTLS_CLIENT_CERT_PATH");
                let mtls_key_path = env::var("PLATUNE_MTLS_CLIENT_KEY_PATH");
                if let (Ok(mtls_cert_path), Ok(mtls_key_path)) = (mtls_cert_path, mtls_key_path) {
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
        // We need to add a header to tell the Icecast server that we can parse the metadata
        // embedded within the stream itself.
        let client = client_builder.request_icy_metadata().build()?;

        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let client = reqwest_middleware::ClientBuilder::new(client)
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

        // live streams have fixed transfer rates so we'll limit prefetch to 2 seconds
        const PREFETCH_SECONDS: u64 = 5;
        const LIVE_PREFETCH_SECONDS: u64 = 2;
        let settings = Settings::default();
        let icy_headers = IcyHeaders::parse_from_headers(stream.headers());
        // radio streams commonly include an Icy-Br header to denote the bitrate
        let prefetch_bytes = if let Some(bitrate) = icy_headers.bitrate() {
            bitrate_to_prefetch(bitrate, Duration::from_secs(LIVE_PREFETCH_SECONDS))
        } else {
            let subtype = &stream
                .content_type()
                .as_ref()
                .map(|t| t.subtype.as_str())
                .unwrap_or("");
            let prefetch_seconds = if file_len.is_some() {
                PREFETCH_SECONDS
            } else {
                LIVE_PREFETCH_SECONDS
            };
            bitrate_to_prefetch(
                content_subtype_to_bitrate(subtype),
                Duration::from_secs(prefetch_seconds),
            )
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
        let token = reader.cancellation_token();
        if let Some(icy_metadata_interval) = icy_headers.metadata_interval() {
            info!("detected icecast metadata. interval: {icy_metadata_interval}");
            let station_name = icy_headers.name().map(|s| s.to_owned());
            let on_track_changed = self.on_track_changed.clone();
            let icy_reader =
                IcyMetadataReader::new(reader, Some(icy_metadata_interval), move |metadata| {
                    if let Ok(metadata) =
                        metadata.inspect_err(|e| warn!("error parsing icy metadata: {e:?}"))
                    {
                        let title = metadata.stream_title();
                        on_track_changed(Metadata {
                            song: title.map(|t| t.to_string()),
                            // TODO: maybe add some custom metadata for radio stations
                            artist: station_name.clone(),
                            ..Default::default()
                        });
                    }
                });
            let track = MetadataSource {
                source: Box::new(ReadSeekSource::new(icy_reader, file_len, extension)),
                metadata: None,
            };
            Ok((track, token))
        } else {
            let track = MetadataSource {
                source: Box::new(ReadSeekSource::new(reader, file_len, extension)),
                metadata: None,
            };

            Ok((track, token))
        }
    }
}

fn bitrate_to_prefetch(mut bitrate: u32, buffer_time: Duration) -> u64 {
    // If bitrate is > 1000, it was probably incorrectly sent as bits/sec instead of kilobits/sec.
    if bitrate > 1000 {
        bitrate /= 1000;
    }
    // buffer 5 seconds of audio
    // bitrate (in kilobits) / bits per byte * bytes per kilobyte * 5 seconds
    (bitrate / 8 * 1000) as u64 * buffer_time.as_secs()
}

fn content_subtype_to_bitrate(subtype: &str) -> u32 {
    match subtype {
        "vorbis" | "opus" | "ogg" => 96,
        "aac" => 128,
        "mpeg" => 256,
        _ => 128,
    }
}

pub(crate) struct FileSourceResolver {
    rules: Vec<Rule>,
}

impl FileSourceResolver {
    pub(crate) fn new() -> Self {
        Self {
            rules: vec![Rule::any_url(), Rule::any_string()],
        }
    }
}

#[async_trait::async_trait]
impl RegistryEntry<Result<(MetadataSource, CancellationToken)>> for FileSourceResolver {
    fn priority(&self) -> u32 {
        3
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    async fn handler(&mut self, input: Input) -> Result<(MetadataSource, CancellationToken)> {
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
        let track = MetadataSource {
            source: Box::new(ReadSeekSource::new(reader, file_len, extension)),
            metadata: None,
        };
        Ok((track, CancellationToken::new()))
    }
}
