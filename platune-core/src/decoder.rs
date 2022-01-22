use crate::{dto::current_time::CurrentTime, source::Source};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use symphonia::core::{
    audio::{Channels, SampleBuffer, SignalSpec},
    codecs::{Decoder as SymphoniaDecoder, DecoderOptions},
    formats::{FormatOptions, FormatReader, Packet, SeekMode, SeekTo, SeekedTo},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
    units::{Time, TimeBase},
};
use tracing::info;

pub(crate) struct Decoder {
    buf: Vec<f64>,
    sample_buf: SampleBuffer<f64>,
    decoder: Box<dyn SymphoniaDecoder>,
    reader: Box<dyn FormatReader>,
    time_base: TimeBase,
    buf_len: usize,
    volume: f64,
    track_id: u32,
    input_channels: usize,
    output_channels: usize,
    timestamp: u64,
    paused: bool,
    sample_rate: usize,
}

impl Decoder {
    pub(crate) fn new(source: Box<dyn Source>, volume: f64, output_channels: usize) -> Self {
        let mut hint = Hint::new();
        if let Some(extension) = source.get_file_ext() {
            hint.with_extension(&extension);
        }
        let mss = MediaSourceStream::new(source.as_media_source(), Default::default());

        let format_opts = FormatOptions {
            enable_gapless: true,
            ..FormatOptions::default()
        };
        let metadata_opts = MetadataOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .unwrap();

        let reader = probed.format;

        let track = reader.default_track().unwrap();

        let time_base = track.codec_params.time_base.unwrap();
        let decode_opts = DecoderOptions { verify: true };
        let symphonia_decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decode_opts)
            .unwrap();
        let track = reader.default_track().unwrap();
        let track_id = track.id;

        let mut decoder = Self {
            decoder: symphonia_decoder,
            reader,
            time_base,
            buf_len: 0,
            input_channels: 0,
            output_channels,
            track_id,
            buf: vec![],
            sample_buf: SampleBuffer::<f64>::new(0, SignalSpec::new(0, Channels::all())),
            volume,
            timestamp: 0,
            paused: false,
            sample_rate: 0,
        };
        decoder.skip_silence();

        decoder
    }

    pub(crate) fn set_volume(&mut self, volume: f64) {
        self.volume = volume;
    }

    pub(crate) fn volume(&self) -> f64 {
        self.volume
    }

    pub(crate) fn pause(&mut self) {
        self.paused = true;
    }

    pub(crate) fn resume(&mut self) {
        self.paused = false;
    }

    pub(crate) fn sample_rate(&self) -> usize {
        self.sample_rate
    }

    pub(crate) fn seek(
        &mut self,
        time: Duration,
    ) -> Result<SeekedTo, symphonia::core::errors::Error> {
        let nanos_per_sec = 1_000_000_000.0;
        self.reader.seek(
            SeekMode::Coarse,
            SeekTo::Time {
                time: Time::new(time.as_secs(), time.subsec_nanos() as f64 / nanos_per_sec),
                track_id: Some(self.track_id),
            },
        )
    }

    pub(crate) fn current_position(&self) -> CurrentTime {
        let time = self.time_base.calc_time(self.timestamp);
        let millis = ((time.seconds as f64 + time.frac) * 1000.0) as u64;

        CurrentTime {
            current_time: Some(Duration::from_millis(millis)),
            retrieval_time: Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap()),
        }
    }

    fn skip_silence(&mut self) {
        let mut samples_skipped = 0;
        loop {
            self.next();
            if let Some(index) = self.buf.iter().position(|s| *s != 0.0) {
                self.buf_len -= index;
                samples_skipped += index;
                let buf_no_silence = self.buf[index..].to_owned();
                self.buf[..self.buf_len].copy_from_slice(&buf_no_silence);

                info!("Skipped {samples_skipped} silent samples");
                break;
            } else {
                samples_skipped += self.buf.len();
            }
        }
    }

    fn adjust_buffer_size(&mut self, samples_length: usize) {
        if samples_length > self.buf.len() {
            self.buf.clear();
            self.buf.resize(samples_length, 0.0);
        }
        self.buf_len = samples_length;
    }

    fn process_output(&mut self, packet: &Packet) {
        let decoded = self.decoder.decode(packet).unwrap();

        if self.sample_rate == 0 {
            let duration = decoded.capacity();
            let spec = *decoded.spec();
            self.sample_rate = spec.rate as usize;
            self.input_channels = spec.channels.count();
            self.sample_buf = SampleBuffer::<f64>::new(duration as u64, spec);
        }
        self.sample_buf.copy_interleaved_ref(decoded);
        let samples_len = self.sample_buf.samples().len();

        match (self.input_channels, self.output_channels) {
            (1, 2) => {
                self.adjust_buffer_size(samples_len * 2);

                let mut i = 0;
                for sample in self.sample_buf.samples().iter() {
                    self.buf[i] = *sample;
                    self.buf[i + 1] = *sample;
                    i += 2;
                }
            }
            (2, 1) => {
                self.adjust_buffer_size(samples_len / 2);

                for (i, sample) in self.sample_buf.samples().chunks_exact(2).enumerate() {
                    self.buf[i] = (sample[0] + sample[1]) / 2.0;
                }
            }
            _ => {
                self.adjust_buffer_size(samples_len);

                for (i, sample) in self.sample_buf.samples().iter().enumerate() {
                    self.buf[i] = *sample;
                }
            }
        }
    }

    pub(crate) fn current(&self) -> &[f64] {
        &self.buf[..self.buf_len]
    }

    pub(crate) fn next(&mut self) -> Option<&[f64]> {
        if self.paused {
            self.buf.fill(0.0);
        } else {
            let packet = loop {
                match self.reader.next_packet() {
                    Ok(packet) => {
                        if packet.track_id() == self.track_id {
                            break packet;
                        }
                    }
                    Err(_) => {
                        return None;
                    }
                };
            };
            self.timestamp = packet.ts();
            self.process_output(&packet);
        }
        Some(self.current())
    }
}
