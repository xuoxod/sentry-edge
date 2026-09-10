//! Sentry Outbound Bridge Client to Conduit Relay Gateway (Self-Contained)

use sentry_core::{SentryAlert, SentryError, SentryResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Sovereign Wire Protocol packet for transmission across Conduit virtual streams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryWirePacket {
    pub stream_id: u32,
    pub seq_num: u64,
    pub payload_json: String,
}

#[derive(Debug, Clone)]
pub struct SentryConduitBridge {
    pub relay_url: String,
    pub auth_token: Option<String>,
    pub node_id: Uuid,
    pub node_label: String,
    pub is_connected: bool,
    pub queued_alerts: Vec<SentryAlert>,
    seq_counter: u64,
}

impl SentryConduitBridge {
    pub fn new(
        relay_url: impl Into<String>,
        auth_token: Option<String>,
        node_id: Uuid,
        node_label: impl Into<String>,
    ) -> Self {
        Self {
            relay_url: relay_url.into(),
            auth_token,
            node_id,
            node_label: node_label.into(),
            is_connected: true,
            queued_alerts: Vec::new(),
            seq_counter: 0,
        }
    }

    /// Frame an alert as a SentryWirePacket to transmit through Conduit virtual multiplexing channels.
    pub fn frame_alert(&mut self, alert: &SentryAlert) -> SentryResult<SentryWirePacket> {
        let payload_json = serde_json::to_string(alert)
            .map_err(|e| SentryError::Serialization(e))?;

        self.seq_counter += 1;

        let packet = SentryWirePacket {
            stream_id: 1, // Channel 1: Telemetry & Physical Security Alerts
            seq_num: self.seq_counter,
            payload_json,
        };

        Ok(packet)
    }

    /// Dispatch an alert to the queue for outward transmission.
    pub fn dispatch_alert(&mut self, alert: SentryAlert) -> SentryResult<SentryWirePacket> {
        let packet = self.frame_alert(&alert)?;
        self.queued_alerts.push(alert);
        Ok(packet)
    }
}
