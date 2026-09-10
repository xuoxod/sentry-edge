//! Sentry Domain Data Models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Severity level of a triggered sentry incident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IncidentSeverity {
    Info,
    Warning,
    High,
    Critical,
}

/// Incident categorization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SentryIncidentType {
    AcousticSpike { peak_db: f32, baseline_db: f32, delta_db: f32 },
    MotionDetected { delta_score: f32 },
    TamperDetected { reason: String },
    ManualPanic { operator: String },
    PeriodicHeartbeat,
}

/// A captured physical security event alert.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SentryAlert {
    pub alert_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub node_id: Uuid,
    pub node_label: String,
    pub severity: IncidentSeverity,
    pub incident: SentryIncidentType,
    pub description: String,
    pub snapshot_frame_base64: Option<String>,
    pub event_signature_sha256: String,
}

impl SentryAlert {
    pub fn new(
        node_id: Uuid,
        node_label: impl Into<String>,
        severity: IncidentSeverity,
        incident: SentryIncidentType,
        description: impl Into<String>,
        snapshot_frame_base64: Option<String>,
    ) -> Self {
        let alert_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let node_label = node_label.into();
        let description = description.into();

        let signature = Self::compute_signature(&alert_id, &timestamp, &node_id, &node_label, &description);

        Self {
            alert_id,
            timestamp,
            node_id,
            node_label,
            severity,
            incident,
            description,
            snapshot_frame_base64,
            event_signature_sha256: signature,
        }
    }

    /// Calculate SHA-256 integrity signature for an alert's immutable fields.
    pub fn compute_signature(
        alert_id: &Uuid,
        timestamp: &DateTime<Utc>,
        node_id: &Uuid,
        node_label: &str,
        description: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(alert_id.as_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        hasher.update(node_id.as_bytes());
        hasher.update(node_label.as_bytes());
        hasher.update(description.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Cryptographically verify if the alert payload has been modified in flight (Anti-Tamper).
    pub fn verify_signature(&self) -> bool {
        let expected = Self::compute_signature(
            &self.alert_id,
            &self.timestamp,
            &self.node_id,
            &self.node_label,
            &self.description,
        );
        self.event_signature_sha256 == expected
    }
}
