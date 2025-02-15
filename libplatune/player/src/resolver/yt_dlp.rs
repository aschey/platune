use std::env;
use std::num::NonZeroUsize;
use std::time::Duration;

use async_trait::async_trait;
use decal::decoder::{ReadSeekSource, Source};
use eyre::{Context, Result, bail};
use lazy_regex::{Lazy, regex};
use regex::Regex;
use stream_download::process::{
    CommandBuilder, FfmpegConvertAudioCommand, ProcessStreamParams, YtDlpCommand,
};
use stream_download::registry::{self, Input, RegistryEntry, Rule};
use stream_download::storage::adaptive::AdaptiveStorageProvider;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use tap::{Tap, TapFallible};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use which::which;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

macro_rules! url_regex {
    ($s:expr) => {
        Lazy::force(regex!($s)).clone()
    };
}

fn audius() -> Regex {
    url_regex!(r"^(www\.)?audius\.co$")
}

fn twitch() -> Regex {
    url_regex!(r"^(www\.)?twitch\.tv$")
}

fn youtube() -> Regex {
    url_regex!(r"^(www\.)?youtube\.com$")
}

fn audiomack() -> Regex {
    url_regex!(r"^(www\.)?audiomack\.com$")
}

fn bandcamp() -> Regex {
    url_regex!(r"^(www\.)?(.*\.)?bandcamp\.com$")
}

fn soundcloud() -> Regex {
    url_regex!(r"^(www\.)?soundcloud.com$")
}

fn globalplayer() -> Regex {
    url_regex!(r"^(www\.)?globalplayer.com$")
}

fn ytdl_rules() -> Vec<Rule> {
    vec![
        Rule::prefix("ytdl://"),
        Rule::http_domain(youtube()),
        Rule::http_domain(twitch()),
        Rule::http_domain(audius()),
        Rule::http_domain(audiomack()),
        Rule::http_domain(bandcamp()),
        Rule::http_domain(soundcloud()),
        Rule::http_domain(globalplayer()),
        // iheart has limited support currently: https://github.com/yt-dlp/yt-dlp/issues/2890
        //Rule::http_domain(url_regex!(r"^(www\.)?iheart.com$")),
        // seems to get stuck on album downloads without --playlist-end
        //Rule::http_domain(url_regex!(r"^(www\.)?last.fm$")),
    ]
}

fn find_exe(env_var: &str, exe_name: &str) -> Result<String> {
    let path =
        env::var(env_var).or_else(|_| which(exe_name).map(|p| p.to_string_lossy().to_string()))?;

    info!("Using {exe_name} path: {path:?}");
    Ok(path)
}

fn ytdl_exe() -> Result<String> {
    find_exe("YT_DLP_PATH", "yt-dlp").tap_err(|e| error!("yt-dlp path not found: {e:?}"))
}

fn ffmpeg_exe() -> Result<String> {
    find_exe("FFMPEG_PATH", "ffmpeg").tap_err(|e| error!("ffmpeg path not found: {e:?}"))
}

struct RegexSet {
    regexes: Vec<Regex>,
}

impl RegexSet {
    fn single(re: Regex) -> Self {
        Self { regexes: vec![re] }
    }

    fn new(regexes: Vec<Regex>) -> Self {
        Self { regexes }
    }

    fn matches(&self, input: &str) -> bool {
        for regex in &self.regexes {
            if regex.is_match(input) {
                return true;
            }
        }
        false
    }
}

pub(crate) struct YtDlpUrlResolver {
    rules: Vec<Rule>,
    skip_flat_playlist: RegexSet,
    force_original_url: RegexSet,
}

impl YtDlpUrlResolver {
    pub(crate) fn new() -> Self {
        Self {
            rules: ytdl_rules(),
            // some sites don't populate urls when using --flat-playlist so we need to explicitly
            // skip it
            skip_flat_playlist: RegexSet::single(audius()),
            // some sites may return a url that's incompatible with our default logic
            force_original_url: RegexSet::new(vec![twitch(), youtube()]),
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
        let source_url = input.source.clone().into_url();
        info!("source url: {source_url}");
        let flat_playlist = !self
            .skip_flat_playlist
            .matches(source_url.domain().unwrap_or_default());
        let mut command = YoutubeDl::new(input.source.clone());
        command.youtube_dl_path(ytdl_exe()?);
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
                if !self
                    .force_original_url
                    .matches(source_url.domain().unwrap_or_default())
                {
                    if let Some(Ok(url)) = video.url.map(|u| u.parse()) {
                        // prefer URL from the command output if available
                        input.source = registry::Source::Url(url);
                    }
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
            .youtube_dl_path(ytdl_exe()?)
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
                let worst_quality = 10.0;
                // find best format (0 is best, 10 is worst)
                formats
                    .into_iter()
                    .filter(|f| {
                        // use native audio codec or format if available
                        if let Some(audio_codec) = &f.acodec {
                            if yt_dlp_formats.contains(&audio_codec.as_str()) {
                                info!("using native audio codec");
                                return true;
                            }
                        }
                        if let Some(format) = &f.format {
                            yt_dlp_formats
                                .contains(&format.as_str())
                                .tap(|found| info!("native format found: {found}"))
                        } else {
                            false
                        }
                    })
                    .reduce(|best, format| {
                        if format.quality.unwrap_or(worst_quality)
                            < best.quality.unwrap_or(worst_quality)
                        {
                            format
                        } else {
                            best
                        }
                    })
            }
            YoutubeDlOutput::Playlist(playlist) => {
                // This shouldn't happen since we're enumerating playlists in the URL resolver
                warn!("found playlist in source resolver: {:?}", playlist.title);
                None
            }
        };
        let cmd = YtDlpCommand::new(input.source)
            .yt_dlp_path(ytdl_exe()?)
            .extract_audio(true);
        let ffmpeg_args = ["--ffmpeg-location", &ffmpeg_exe()?];
        let params = if let Some(format) = &found_format {
            info!("source quality: {:?}", format.quality);
            info!("source is in an appropriate format, no post-processing required");
            // Prefer the explicit format ID since this insures the format used will match
            // the filesize.
            let format_id = format.format_id.clone().expect("format id missing");
            let params =
                ProcessStreamParams::new(cmd.format(format_id).into_command().args(ffmpeg_args))?;
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
            let builder = CommandBuilder::new(cmd.into_command().args(ffmpeg_args))
                .pipe(FfmpegConvertAudioCommand::new(ffmpeg_format).ffmpeg_path(ffmpeg_exe()?));
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
        let token = reader.cancellation_token();

        Ok((
            Box::new(ReadSeekSource::new(reader, size, Some("m4a".to_string()))),
            token,
        ))
    }
}
