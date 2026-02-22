use std::time::Duration;

use decal::decoder::{CurrentPosition, Decoder, DecoderResult};
use decal::output::Host;
use decal::symphonia::core::meta::{MetadataRevision, StandardTag};
use decal::{AudioManager, ResetMode};
use flume::TryRecvError;
use tap::TapFallible;
use tracing::{error, info, warn};

use crate::dto::decoder_command::DecoderCommand;
use crate::dto::decoder_response::DecoderResponse;
use crate::dto::processor_error::ProcessorError;
use crate::platune_player::{Metadata, PlayerEvent, SeekMode};
use crate::two_way_channel::TwoWayReceiver;

pub(crate) struct AudioProcessor<'a, H: Host> {
    cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
    manager: &'a mut AudioManager<f32, H>,
    decoder: Decoder<f32>,
    last_sent_position: Duration,
    event_tx: &'a tokio::sync::broadcast::Sender<PlayerEvent>,
    input_metadata: Option<Metadata>,
    metadata_init: bool,
}

pub(crate) enum InputResult {
    Continue,
    Stop,
}

macro_rules! find_tag {
    ($tags:expr, $tag_type:path) => {
        $tags
            .iter()
            .filter_map(|t| {
                if let $tag_type(val) = t {
                    Some(val.to_string())
                } else {
                    None
                }
            })
            .next()
    };
}

impl<'a, H: Host> AudioProcessor<'a, H> {
    pub(crate) fn new(
        manager: &'a mut AudioManager<f32, H>,
        decoder: Decoder<f32>,
        cmd_rx: &'a mut TwoWayReceiver<DecoderCommand, DecoderResponse>,
        event_tx: &'a tokio::sync::broadcast::Sender<PlayerEvent>,
        input_metadata: Metadata,
    ) -> Result<Self, ProcessorError> {
        match cmd_rx.recv() {
            Ok(DecoderCommand::WaitForInitialization) => {
                info!("Notifying decoder started");
            }
            Ok(cmd) => {
                error!("Got unexpected command {cmd:?}");
            }
            Err(e) => {
                error!("Error receiving initialization message {e:?}");
            }
        }

        cmd_rx
            .respond(DecoderResponse::InitializationSucceeded)
            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
            .tap_err(|e| error!("Error sending decoder initialization succeeded: {e:?}"))?;

        Ok(Self {
            decoder,
            manager,
            cmd_rx,
            event_tx,
            input_metadata: Some(input_metadata),
            metadata_init: false,
            last_sent_position: Duration::ZERO,
        })
    }

    fn process_input(&mut self) -> Result<InputResult, ProcessorError> {
        match self.cmd_rx.try_recv() {
            Ok(command) => {
                info!("Got decoder command {:?}", command);

                match command {
                    DecoderCommand::Play => {
                        self.decoder.resume();

                        self.cmd_rx
                            .respond(DecoderResponse::Received)
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| error!("Error sending stopped response: {e:?}"))?;
                    }
                    DecoderCommand::Stop => {
                        self.cmd_rx
                            .respond(DecoderResponse::Received)
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| error!("Error sending stopped response: {e:?}"))?;

                        return Ok(InputResult::Stop);
                    }
                    DecoderCommand::Seek(time, mode) => {
                        let current_time = self.decoder.current_position();
                        let seek_time = match mode {
                            SeekMode::Absolute => time,
                            SeekMode::Forward => current_time.position + time,
                            SeekMode::Backward => current_time.position - time,
                        };
                        let seek_response = match self.decoder.seek(seek_time) {
                            Ok(seeked_to) => Ok(seeked_to.actual_ts),
                            Err(e) => Err(e.to_string()),
                        };

                        self.cmd_rx
                            .respond(DecoderResponse::SeekResponse(seek_response))
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| error!("Unable to send seek result: {e:?}"))?;
                    }
                    DecoderCommand::Pause => {
                        self.decoder.pause();
                        self.manager.pause();

                        self.cmd_rx
                            .respond(DecoderResponse::Received)
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| error!("Error sending stopped response: {e:?}"))?;
                    }
                    DecoderCommand::SetVolume(volume) => {
                        self.manager.set_volume(volume);
                        self.decoder.set_volume(volume);

                        self.cmd_rx
                            .respond(DecoderResponse::Received)
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| error!("Error sending set volume response: {e:?}"))?;
                    }
                    DecoderCommand::GetCurrentPosition => {
                        let time = self.decoder.current_position();

                        self.cmd_rx
                            .respond(DecoderResponse::CurrentPositionResponse(time))
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| {
                                error!("Unable to send current position response: {e:?}")
                            })?;
                    }
                    DecoderCommand::Reset => {
                        self.reset();
                        self.cmd_rx
                            .respond(DecoderResponse::Received)
                            .map_err(|e| ProcessorError::CommunicationError(format!("{e:?}")))
                            .tap_err(|e| error!("Error sending set volume response: {e:?}"))?;
                    }
                    DecoderCommand::WaitForInitialization => {
                        unreachable!("Should only send this during initialization");
                    }
                }
                info!("Completed decoder command");
            }
            Err(TryRecvError::Empty) => {
                let position = self.decoder.current_position();
                // if position.position < last_send_time, we just seeked backwards
                if position.position < self.last_sent_position
                    || position.position - self.last_sent_position >= Duration::from_secs(10)
                {
                    self.event_tx
                        .send(PlayerEvent::Position(position.clone()))
                        .unwrap_or_default();
                    self.last_sent_position = position.position;
                }
            }
            Err(TryRecvError::Disconnected) => {
                info!("Decoder command sender has disconnected");
                return Ok(InputResult::Stop);
            }
        }

        Ok(InputResult::Continue)
    }

    pub(crate) fn position(&self) -> CurrentPosition {
        self.decoder.current_position()
    }

    pub(crate) fn next(&mut self) -> Result<(InputResult, DecoderResult), ProcessorError> {
        match self.process_input() {
            Ok(InputResult::Continue) => {}
            Ok(InputResult::Stop) => return Ok((InputResult::Stop, DecoderResult::Finished)),
            Err(e) => return Err(e),
        };
        let res = self
            .manager
            .write(&mut self.decoder)
            .map_err(ProcessorError::WriteOutputError)?;
        Ok((InputResult::Continue, res))
    }

    pub(crate) fn reset(&mut self) {
        // Reset may fail on Windows on the first try if the device was unplugged
        let _ = self
            .manager
            .reset(&mut self.decoder, ResetMode::Force)
            .inspect_err(|e| warn!("error resetting {e:?}"));
    }

    pub(crate) fn next_metadata(&mut self) -> Option<Metadata> {
        if let Some(input_metadata) = self.input_metadata.take() {
            let latest = self.decoder.metadata().skip_to_latest().cloned();
            self.metadata_init = true;
            // Prefer the metadata from the decoder if the input metadata only has the default
            // title attached
            if input_metadata.artist.is_none()
                && input_metadata.album_artist.is_none()
                && let Some(latest) = latest
            {
                return Some(self.extract_metadata(&latest));
            }
            return Some(input_metadata);
        }

        let mut metadata = self.decoder.metadata();
        if (!self.metadata_init || !metadata.is_latest())
            && let Some(latest) = metadata.skip_to_latest().cloned()
        {
            self.metadata_init = true;
            Some(self.extract_metadata(&latest))
        } else {
            None
        }
    }

    fn extract_metadata(&self, latest: &MetadataRevision) -> Metadata {
        let track_id = self.decoder.track_id() as u64;
        let per_track = latest
            .per_track
            .iter()
            .find_map(|t| {
                if t.track_id == track_id {
                    Some(t.metadata.tags.iter())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let std_tags: Vec<_> = per_track
            .into_iter()
            .chain(latest.media.tags.iter())
            .filter_map(|t| t.std.as_ref())
            .collect();
        Metadata {
            artist: find_tag!(std_tags, StandardTag::Artist),
            album_artist: find_tag!(std_tags, StandardTag::AlbumArtist),
            album: find_tag!(std_tags, StandardTag::Album),
            song: find_tag!(std_tags, StandardTag::TrackTitle),
            track_number: find_tag!(std_tags, StandardTag::TrackNumber)
                .map(|t| t.parse().ok())
                .flatten(),
            duration: self.decoder.duration(),
        }
    }
}
