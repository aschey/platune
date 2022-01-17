use crate::{dto::current_time::CurrentTime, source::Source};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use symphonia::core::{
    audio::{SampleBuffer, SignalSpec},
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
    spec: SignalSpec,
    position: usize,
    track_id: u32,
    input_channels: usize,
    output_channels: usize,
    timestamp: u64,
    paused: bool,
}

impl Decoder {
    pub(crate) fn new(source: Box<dyn Source>, output_channels: usize) -> Self {
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

        let mut reader = probed.format;

        let track = reader.default_track().unwrap();

        let time_base = track.codec_params.time_base.unwrap();
        let decode_opts = DecoderOptions { verify: true };
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decode_opts)
            .unwrap();
        let track = reader.default_track().unwrap();
        let track_id = track.id;

        let (spec, sample_buf, position, timestamp) =
            Self::process_first_packet(&mut reader, &mut decoder, track_id);
        let buf = sample_buf.samples().to_owned();

        Self {
            decoder,
            reader,
            time_base,
            buf_len: buf.len(),
            input_channels: spec.channels.count(),
            output_channels,
            track_id,
            buf,
            sample_buf,
            spec,
            position,
            timestamp,
            paused: false,
        }
    }

    pub(crate) fn pause(&mut self) {
        self.paused = true;
    }

    pub(crate) fn resume(&mut self) {
        self.paused = false;
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.spec.rate
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

    fn process_first_packet(
        reader: &mut Box<dyn FormatReader>,
        decoder: &mut Box<dyn SymphoniaDecoder>,
        track_id: u32,
    ) -> (SignalSpec, SampleBuffer<f64>, usize, u64) {
        let mut skipped_samples = 0;
        loop {
            match reader.next_packet() {
                Ok(packet) => {
                    if packet.track_id() != track_id {
                        continue;
                    }
                    match decoder.decode(&packet) {
                        Ok(decoded) => {
                            let duration = decoded.capacity();
                            let spec = *decoded.spec();
                            let mut sample_buf = SampleBuffer::<f64>::new(duration as u64, spec);
                            sample_buf.copy_interleaved_ref(decoded);
                            let position: usize;
                            let samples = sample_buf.samples();
                            if let Some(index) = samples.iter().position(|s| *s != 0.0) {
                                skipped_samples += index;
                                info!("Skipped {} silent samples", skipped_samples);
                                position = index;
                            } else {
                                skipped_samples += samples.len();
                                continue;
                            }

                            return (spec, sample_buf, position, packet.ts());
                        }
                        Err(e) => {
                            continue;
                        }
                    }
                }
                Err(e) => {}
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
        // Audio samples must be interleaved for cpal. Interleave the samples in the audio
        // buffer into the sample buffer.
        let decoded = self.decoder.decode(packet).unwrap();
        self.sample_buf.copy_interleaved_ref(decoded);
        // Write all the interleaved samples to the ring buffer.
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
}

impl Iterator for Decoder {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.buf_len {
            let ret = Some(self.buf[self.position]);
            self.position += 1;
            return ret;
        }

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

        self.position = 1;

        Some(self.buf[0])
    }
}
