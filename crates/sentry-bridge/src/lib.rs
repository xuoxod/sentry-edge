//! # Sentry Bridge (`sentry-bridge`)
//! Outbound TLS WebSocket connector linking Sentry to the Conduit Sovereign Relay.

pub mod client;

pub use client::SentryConduitBridge;
