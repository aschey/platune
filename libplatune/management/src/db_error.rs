use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error(transparent)]
    DbError(#[from] sqlx::Error),
    #[error(transparent)]
    MigrateError(#[from] MigrateError),
    #[error("Error loading spellfix: {0}")]
    SpellfixLoadError(String),
}
