use thiserror::Error;

#[derive(Error, Debug)]
pub enum TelemetryError {
    #[error("I/O error in telemetry sink: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("SQLite ledger error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Log injection attack detected: {0}")]
    LogInjection(String),

    #[error("Hash chain integrity broken at sequence #{sequence}: expected prev_hash '{expected}', got '{actual}'")]
    HashChainBroken {
        sequence: u64,
        expected: String,
        actual: String,
    },

    #[error("Telemetry engine error: {0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, TelemetryError>;
