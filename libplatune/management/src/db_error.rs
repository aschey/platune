use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DbError {
    #[error("Database error: {0}")]
    DbError(String),
    #[error("Migration error: {0}")]
    MigrateError(String),
    #[error("Error loading spellfix: {0}")]
    SpellfixLoadError(String),
}
