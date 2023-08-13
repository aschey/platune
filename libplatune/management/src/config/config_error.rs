use std::io::Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Unable to locate a valid home directory")]
    NoHomeDir,
    #[error("Failed to create file {0}: {1}")]
    FileCreationFailed(String, Error),
    #[error("{0} is not a file")]
    NotAFile(String),
    #[error("{0} contains invalid unicode")]
    InvalidUnicode(String),
}
