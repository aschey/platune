use std::cmp::Ordering;
use std::collections::HashSet;
use std::env;
use std::num::NonZeroUsize;
use std::time::Duration;

use async_trait::async_trait;
use decal::decoder::{ReadSeekSource, Source};
use eyre::{Context, Result, bail};
use lazy_regex::{Lazy, regex};
use stream_download::process::{
    CommandBuilder, FfmpegConvertAudioCommand, ProcessStreamParams, YtDlpCommand,
};
use stream_download::registry::{self, Input, RegistryEntry, Rule};
use stream_download::storage::adaptive::AdaptiveStorageProvider;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use tap::TapFallible;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

macro_rules! url_regex {
    ($s:expr) => {
        Lazy::force(regex!($s)).clone()
    };
}

fn ytdl_rules() -> Vec<Rule> {
    vec![
        Rule::prefix("ytdl://"),
        Rule::http_domain(url_regex!(r"^(www\.)?youtube\.com$")),
        Rule::http_domain(url_regex!(r"^(www\.)?twitch\.tv$")),
        Rule::http_domain(url_regex!(r"^(www\.)?audius\.co$")),
        Rule::http_domain(url_regex!(r"^(www\.)?audiomack\.com$")),
        Rule::http_domain(url_regex!(r"^(www\.)?(.*\.)?bandcamp\.com$")),
        Rule::http_domain(url_regex!(r"^(www\.)?soundcloud.com$")),
        Rule::http_domain(url_regex!(r"^(www\.)?globalplayer.com$")),
        // iheart has limited support currently: https://github.com/yt-dlp/yt-dlp/issues/2890
        //Rule::http_domain(url_regex!(r"^(www\.)?iheart.com$")),
        // seems to get stuck on album downloads without --playlist-end
        //Rule::http_domain(url_regex!(r"^(www\.)?last.fm$")),
    ]
}

fn ytdl_exe() -> String {
    let path = env::var("YT_DLP_PATH").unwrap_or_else(|_| "yt-dlp".to_string());
    info!("Using yt-dlp path: {path:?}");
    path
}

pub(crate) struct YtDlpUrlResolver {
    rules: Vec<Rule>,
    skip_flat_playlist: HashSet<&'static str>,
}

impl YtDlpUrlResolver {
    pub(crate) fn new() -> Self {
        let mut skip = HashSet::new();
        // some sites don't populate urls when using --flat-playlist so we need to explicitly skip
        // it
        skip.insert("audius.co");
        Self {
            rules: ytdl_rules(),
            skip_flat_playlist: skip,
        }
    }
}

#[async_trait]
impl RegistryEntry<Result<Vec<Input>>> for YtDlpUrlResolver {
    fn priority(&self) -> u32 {
        1
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    async fn handler(&mut self, mut input: Input) -> Result<Vec<Input>> {
        info!("extracting video metadata - this may take a few seconds");
        let flat_playlist = !self
            .skip_flat_playlist
            .contains(input.source.clone().into_url().domain().unwrap_or_default());
        let mut command = YoutubeDl::new(input.source.clone());
        command.youtube_dl_path(ytdl_exe());
        if flat_playlist {
            // --flat-playlist prevents it from enumerating all videos in the playlist, which could
            // take a long time
            command.extra_arg("--flat-playlist");
        }
        let output = command.run_async().await.wrap_err("error running yt-dlp")?;
        info!("metadata extraction complete");

        match output {
            YoutubeDlOutput::SingleVideo(video) => {
                info!("found single video: {:?}", video.title);
                info!("url {:?}", video.url);
                if let Some(Ok(url)) = video.url.map(|u| u.parse()) {
                    // prefer URL from the command output if available
                    input.source = registry::Source::Url(url);
                }
                Ok(vec![input])
            }
            YoutubeDlOutput::Playlist(playlist) => {
                info!("found playlist: {:?}", playlist.title);
                Ok(playlist
                    .entries
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|entry| {
                        let Ok(url) = entry
                            .url
                            .clone()
                            .unwrap_or_default()
                            .parse()
                            .tap_err(|e| error!("error parsing url {:?}: {e:?}", entry.url))
                        else {
                            return None;
                        };
                        Some(Input {
                            prefix: input.prefix.clone(),
                            source: registry::Source::Url(url),
                        })
                    })
                    .collect())
            }
        }
    }
}

pub(crate) struct YtDlpSourceResolver {
    rules: Vec<Rule>,
}

impl YtDlpSourceResolver {
    pub(crate) fn new() -> Self {
        Self {
            rules: ytdl_rules(),
        }
    }
}

#[async_trait]
impl RegistryEntry<Result<(Box<dyn Source>, CancellationToken)>> for YtDlpSourceResolver {
    fn priority(&self) -> u32 {
        1
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    async fn handler(&mut self, input: Input) -> Result<(Box<dyn Source>, CancellationToken)> {
        let yt_dlp_formats = ["m4a", "mp3"];
        let ffmpeg_format = "adts";

        info!("ytdl video url: {}", input.source);
        info!("extracting video metadata - this may take a few seconds");
        let output = YoutubeDl::new(input.source.clone())
            .extract_audio(true)
            .run_async()
            .await?;
        info!("metadata extraction complete");

        let found_format = match output {
            YoutubeDlOutput::SingleVideo(video) => {
                info!("found single video: {:?}", video.title);
                let Some(formats) = video.formats else {
                    bail!("No formats found");
                };
                let mut valid_formats: Vec<_> = formats
                    .into_iter()
                    .filter(|f| {
                        if let Some(format) = &f.format {
                            yt_dlp_formats.contains(&format.as_str())
                        } else {
                            false
                        }
                    })
                    .collect();
                // Sort formats by quality (0 is best, 10 is worst)
                valid_formats
                    .sort_by(|a, b| a.quality.partial_cmp(&b.quality).unwrap_or(Ordering::Equal));
                // Use the best quality one
                valid_formats.pop()
            }
            YoutubeDlOutput::Playlist(playlist) => {
                // This shouldn't happen since we're enumerating playlists in the URL resolver
                warn!("found playlist in source resolver: {:?}", playlist.title);
                None
            }
        };
        let cmd = YtDlpCommand::new(input.source)
            .yt_dlp_path(ytdl_exe())
            .extract_audio(true);

        let params = if let Some(format) = &found_format {
            info!("source quality: {:?}", format.quality);
            info!("source is in an appropriate format, no post-processing required");
            // Prefer the explicit format ID since this insures the format used will match
            // the filesize.
            let format_id = format.format_id.clone().expect("format id missing");
            let params = ProcessStreamParams::new(cmd.format(format_id))?;
            if let Some(size) = format.filesize {
                info!("found video size: {size}");
                params.content_length(size as u64)
            } else {
                params
            }
        } else {
            info!("source requires post-processing - converting to m4a using ffmpeg");
            // yt-dlp can handle format conversion, but if we want to stream it directly from
            // stdout, we have to pipe the raw output to ffmpeg ourselves.
            let builder =
                CommandBuilder::new(cmd).pipe(FfmpegConvertAudioCommand::new(ffmpeg_format));
            ProcessStreamParams::new(builder)?
        };
        let size = found_format.and_then(|f| f.filesize).map(|f| f as u64);

        // Sometimes it may take a while for ffmpeg to output a new chunk, so we can bump up the
        // retry timeout to be safe.
        let settings = Settings::default()
            .retry_timeout(Duration::from_secs(30))
            .cancel_on_drop(false);
        let reader = StreamDownload::new_process(
            params,
            AdaptiveStorageProvider::new(
                TempStorageProvider::with_prefix("platune_cache"),
                // ensure we have enough buffer space to store the prefetch data
                NonZeroUsize::new((settings.get_prefetch_bytes() * 2) as usize)
                    .expect("invalid prefetch bytes"),
            ),
            settings,
        )
        .await?;
        let token = reader.get_cancellation_token();

        Ok((
            Box::new(ReadSeekSource::new(reader, size, Some("m4a".to_string()))),
            token,
        ))
    }
}
