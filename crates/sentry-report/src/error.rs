//! Sentry Report Engine Error Definitions

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReportError {
    #[error("I/O error during report generation: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("SQLite ledger query error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Unknown or unsupported report format: '{0}' (supported: html, json, jsonl, csv, txt, md)")]
    InvalidFormat(String),

    #[error("Empty dataset: cannot generate meaningful report from 0 records")]
    EmptyDataset,

    #[error("Cryptographic hash chain integrity check failed during report generation")]
    HashChainTampered,

    #[error("Core error: {0}")]
    Core(#[from] sentry_core::SentryError),
}

pub type Result<T> = std::result::Result<T, ReportError>;
