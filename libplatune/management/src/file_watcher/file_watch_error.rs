use crate::db_error::DbError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileWatchError {
    #[error(transparent)]
    WatchError(#[from] notify::Error),
    #[error(transparent)]
    DbError(#[from] DbError),
    #[error("Thread communication error: {0}")]
    ThreadCommError(String),
}
