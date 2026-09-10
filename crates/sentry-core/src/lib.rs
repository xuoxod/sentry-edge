//! # Sentry Core (`sentry-core`)
//! Pure Micro-OJP domain logic for the Sovereign Sentry Sentinel.

pub mod acoustic;
pub mod config;
pub mod error;
pub mod models;

pub use acoustic::AcousticAnalyzer;
pub use config::SentryConfig;
pub use error::{SentryError, SentryResult};
pub use models::{IncidentSeverity, SentryAlert, SentryIncidentType};
