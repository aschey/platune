use decal::WriteOutputError;
use thiserror::Error;

#[derive(Error, Debug)]
pub(crate) enum ProcessorError {
    #[error("{0:?}")]
    CommunicationError(String),
    #[error(transparent)]
    WriteOutputError(#[from] WriteOutputError),
}
