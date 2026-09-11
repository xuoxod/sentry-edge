//! # Sentry Ledger (`sentry-ledger`)
//! Enterprise SQLite WAL Incident Persistence Engine with nanosecond commit transactions.

pub mod db;

pub use db::SentryLedgerDb;
