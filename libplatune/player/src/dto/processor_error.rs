use decal::decoder::DecoderError;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ProcessorError {
    #[error("{0:?}")]
    CommunicationError(String),
    #[error(transparent)]
    DecoderError(#[from] DecoderError),
}
