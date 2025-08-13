use std::env;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use decal::decoder::ReadSeekSource;
use eyre::{Context, Result, bail};
use lazy_regex::{Lazy, regex};
use regex::Regex;
use stream_download::process::{
    CommandBuilder, FfmpegConvertAudioCommand, ProcessStreamParams, YtDlpCommand,
};
use stream_download::registry::{self, Input, RegistryEntry, Rule, Source};
use stream_download::storage::adaptive::AdaptiveStorageProvider;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use tap::TapFallible;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use which::which;
use youtube_dl::{YoutubeDl, YoutubeDlOutput};

use super::MetadataSource;
use crate::dto::track::Metadata;
use crate::resolver::HttpSourceResolver;

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
                    && let Some(Ok(url)) = video.url.map(|u| u.parse())
                {
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

const LIBFDK_AAC: &str = "libfdk_aac";

pub(crate) struct YtDlpSourceResolver {
    rules: Vec<Rule>,
    has_fdk_aac: Option<bool>,
    use_direct_url: RegexSet,
    http_resolver: HttpSourceResolver,
}

impl YtDlpSourceResolver {
    pub(crate) fn new() -> Self {
        Self {
            rules: ytdl_rules(),
            has_fdk_aac: None,
            use_direct_url: RegexSet::new(vec![bandcamp(), soundcloud(), audius(), audiomack()]),
            http_resolver: HttpSourceResolver::new(Arc::new(|_| {})),
        }
    }

    async fn has_fdk_aac(&mut self) -> Result<bool> {
        // libfdk_aac is better than the default encoder for ffmpeg, but it isn't included in most
        // distributions
        if let Some(has_fdk_aac) = self.has_fdk_aac {
            Ok(has_fdk_aac)
        } else {
            let ffmpeg_path = ffmpeg_exe()?;
            let output = tokio::process::Command::new(&ffmpeg_path)
                .args(["-v", "quiet", "-codecs"])
                .output()
                .await?;
            let re = Regex::new(&format!("encoders.*{LIBFDK_AAC}"))?;
            let out_str = String::from_utf8(output.stdout)?;
            let has_fdk_aac = re.is_match(&out_str);
            self.has_fdk_aac = Some(has_fdk_aac);
            Ok(has_fdk_aac)
        }
    }
}

#[async_trait]
impl RegistryEntry<Result<(MetadataSource, CancellationToken)>> for YtDlpSourceResolver {
    fn priority(&self) -> u32 {
        1
    }

    fn rules(&self) -> &[Rule] {
        &self.rules
    }

    async fn handler(&mut self, input: Input) -> Result<(MetadataSource, CancellationToken)> {
        let yt_dlp_formats = ["m4a", "mp4a", "mp3"];
        let ffmpeg_format = "adts";
        info!("ytdl video url: {}", input.source);
        info!("extracting video metadata - this may take a few seconds");
        let output = YoutubeDl::new(input.source.clone())
            .youtube_dl_path(ytdl_exe()?)
            .extract_audio(true)
            .run_async()
            .await?;
        info!("metadata extraction complete");

        let (found_format, video) = match output {
            YoutubeDlOutput::SingleVideo(video) => {
                info!("video url: {:?}", video.url);
                if let Some(url) = &video.url {
                    let input_source = input.source.clone().into_url();
                    if self
                        .use_direct_url
                        .matches(input_source.domain().unwrap_or_default())
                    {
                        // For some sites, the direct audio URL can be downloaded with the default
                        // HTTP resolver This is preferable, since it allows
                        // us to have proper seek support
                        info!("using http source resolver");
                        return self
                            .http_resolver
                            .handler(Input {
                                prefix: None,
                                source: Source::Url(url.parse().unwrap()),
                            })
                            .await;
                    }
                }
                info!("found single video: {:?}", video.title);
                let Some(formats) = &video.formats else {
                    bail!("No formats found");
                };
                let worst_quality = 10.0;
                // find best format (0 is best, 10 is worst)
                let format = formats
                    .iter()
                    .filter(|f| {
                        // If no high-quality format is available, it's better to parse it from the
                        // raw input.
                        if f.quality.map(|q| q > 3.0).unwrap_or(false) {
                            info!("quality too low, ignoring: {:?}", f.quality);
                            return false;
                        }
                        // use native audio codec or format if available
                        if let Some(audio_codec) = &f.acodec {
                            info!("checking audio codec: {audio_codec}");
                            if yt_dlp_formats.iter().any(|f| audio_codec.starts_with(f)) {
                                info!(
                                    "using native audio codec {audio_codec} quality: {:?}",
                                    f.quality
                                );
                                return true;
                            }
                        }
                        if let Some(format) = &f.format {
                            info!("checking format: {format} quality: {:?}", f.quality);
                            if yt_dlp_formats.iter().any(|f| format.starts_with(f)) {
                                info!("using native format: {format} quality: {:?}", f.quality);
                                return true;
                            }
                        }
                        false
                    })
                    .reduce(|best, format| {
                        if format.quality.unwrap_or(worst_quality)
                            < best.quality.unwrap_or(worst_quality)
                        {
                            format
                        } else {
                            best
                        }
                    });
                (format.cloned(), video)
            }
            YoutubeDlOutput::Playlist(playlist) => {
                // This shouldn't happen since we're enumerating playlists in the URL resolver
                bail!("found playlist in source resolver: {:?}", playlist.title);
            }
        };

        let metadata = Metadata {
            artist: video.artist,
            album_artist: video.album_artist,
            album: video.album,
            song: video.track.or(video.title),
            track_number: video.track_number.map(|t| t as u32),
            duration: video
                .duration
                .and_then(|d| d.as_f64())
                .map(|d| Duration::from_secs(d as u64)),
        };
        let cmd = YtDlpCommand::new(input.source)
            .yt_dlp_path(ytdl_exe()?)
            .extract_audio(true);
        let ffmpeg_args = ["--ffmpeg-location", &ffmpeg_exe()?];
        let params = if let Some(format) = &found_format {
            info!("source format: {:?}", format.format);
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
            let mut ffmpeg_converter =
                FfmpegConvertAudioCommand::new(ffmpeg_format).ffmpeg_path(ffmpeg_exe()?);

            if self.has_fdk_aac().await? {
                info!("using libfdk_aac");
                ffmpeg_converter = ffmpeg_converter.args(["-c:a", LIBFDK_AAC]);
            } else {
                info!("libfdk_aac not supported, using default aac encoder");
            }
            // yt-dlp can handle format conversion, but if we want to stream it directly from
            // stdout, we have to pipe the raw output to ffmpeg ourselves.
            let builder =
                CommandBuilder::new(cmd.into_command().args(ffmpeg_args)).pipe(ffmpeg_converter);
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
        let track = MetadataSource {
            source: Box::new(ReadSeekSource::new(reader, size, Some("m4a".to_string()))),
            metadata: Some(metadata),
        };
        Ok((track, token))
    }
}
