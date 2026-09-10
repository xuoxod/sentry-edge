//! Sentry Core Error Types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SentryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Hardware capture error: {0}")]
    Hardware(String),

    #[error("Bridge transport error: {0}")]
    Bridge(String),

    #[error("Database ledger error: {0}")]
    Ledger(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type SentryResult<T> = Result<T, SentryError>;
