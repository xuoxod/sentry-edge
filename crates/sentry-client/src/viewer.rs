//! Sentry End-User Remote Viewer & Telepresence Controller

use colored::*;
use sentry_core::{IncidentSeverity, SentryAlert, SentryIncidentType, SentryResult};
use sentry_telepresence::SentryLiveKitSession;

#[derive(Debug, Clone)]
pub struct SentryViewer {
    pub relay_url: String,
    pub auth_token: Option<String>,
    pub operator_label: String,
    pub monitored_nodes: Vec<String>,
    pub received_alerts: Vec<SentryAlert>,
}

impl SentryViewer {
    pub fn new(
        relay_url: impl Into<String>,
        auth_token: Option<String>,
        operator_label: impl Into<String>,
    ) -> Self {
        Self {
            relay_url: relay_url.into(),
            auth_token,
            operator_label: operator_label.into(),
            monitored_nodes: Vec::new(),
            received_alerts: Vec::new(),
        }
    }

    /// Add an edge node to the active watch list.
    pub fn watch_node(&mut self, node_label: impl Into<String>) {
        self.monitored_nodes.push(node_label.into());
    }

    /// Ingest an incoming real-time security alert dispatched from a remote sentinel node.
    pub fn ingest_incoming_alert(&mut self, alert: SentryAlert) -> String {
        let formatted = self.format_alert_banner(&alert);
        self.received_alerts.push(alert);
        formatted
    }

    /// Format an alert into a high-visibility terminal card.
    pub fn format_alert_banner(&self, alert: &SentryAlert) -> String {
        let severity_badge = match alert.severity {
            IncidentSeverity::Critical => "[CRITICAL]".red().bold().to_string(),
            IncidentSeverity::High => "[HIGH]".yellow().bold().to_string(),
            IncidentSeverity::Warning => "[WARN]".yellow().to_string(),
            IncidentSeverity::Info => "[INFO]".cyan().to_string(),
        };

        let type_desc = match &alert.incident {
            SentryIncidentType::AcousticSpike { peak_db, baseline_db: _, delta_db } => {
                format!("Acoustic Spike (+{:.1} dB over baseline | Peak: {:.1} dB)", delta_db, peak_db)
            }
            SentryIncidentType::MotionDetected { delta_score } => {
                format!("V4L2 Camera Motion Detected (Confidence: {:.0}%)", delta_score * 100.0)
            }
            SentryIncidentType::TamperDetected { reason } => {
                format!("Physical Device Tamper ({})", reason)
            }
            SentryIncidentType::ManualPanic { operator } => {
                format!("Manual Operator Panic Triggered by {}", operator)
            }
            SentryIncidentType::PeriodicHeartbeat => "Periodic Sentinel Heartbeat".to_string(),
        };

        format!(
            "{} [{}] 🚨 Node: {} >> {}\n   Description: {}\n   Integrity SHA-256: {}",
            severity_badge,
            alert.timestamp.format("%H:%M:%S"),
            alert.node_label.green().bold(),
            type_desc.bold(),
            alert.description,
            alert.event_signature_sha256[..16].dimmed()
        )
    }

    /// Initiate an on-demand remote telepresence session (sub-10ms WebRTC SFU video/intercom).
    pub fn connect_telepresence(
        &self,
        sfu_url: &str,
        target_node: &str,
    ) -> SentryResult<SentryLiveKitSession> {
        let room_name = format!("sentry-room-{}", target_node.to_lowercase().replace(' ', "-"));
        let session = SentryLiveKitSession::new(sfu_url, room_name, &self.operator_label);
        Ok(session)
    }
}
