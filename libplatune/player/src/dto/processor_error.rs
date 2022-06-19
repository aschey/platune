use thiserror::Error;

use super::decoder_error::DecoderError;

#[derive(Error, Debug)]
pub(crate) enum ProcessorError {
    #[error("{0:?}")]
    CommunicationError(String),
    #[error(transparent)]
    DecoderError(#[from] DecoderError),
}
