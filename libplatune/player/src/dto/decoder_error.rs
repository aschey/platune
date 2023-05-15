use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum DecoderError {
    #[error("No tracks were found")]
    NoTracks,
    #[error("No readable format was discovered: {0}")]
    FormatNotFound(symphonia::core::errors::Error),
    #[error("The codec is unsupported: {0}")]
    UnsupportedCodec(symphonia::core::errors::Error),
    #[error("The format is unsupported: {0}")]
    UnsupportedFormat(String),
    #[error("Error occurred during decoding: {0}")]
    DecodeError(symphonia::core::errors::Error),
    #[error("Recoverable error: {0}")]
    Recoverable(&'static str),
}
