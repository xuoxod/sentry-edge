use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::error::{TelemetryError, Result};
use crate::sanitizer::LogSanitizer;
use crate::time::NanoTimestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubsystemTag {
    AudioDsp,
    CameraV4l2,
    BridgeWs,
    LedgerSqlite,
    IntercomWebrtc,
    AuthGate,
    ClientViewer,
    SystemHeartbeat,
    SystemMaintenance,
}

impl SubsystemTag {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubsystemTag::AudioDsp => "AUDIO_DSP",
            SubsystemTag::CameraV4l2 => "CAMERA_V4L2",
            SubsystemTag::BridgeWs => "BRIDGE_WS",
            SubsystemTag::LedgerSqlite => "LEDGER_SQLITE",
            SubsystemTag::IntercomWebrtc => "INTERCOM_WEBRTC",
            SubsystemTag::AuthGate => "AUTH_GATE",
            SubsystemTag::ClientViewer => "CLIENT_VIEWER",
            SubsystemTag::SystemHeartbeat => "SYSTEM_HEARTBEAT",
            SubsystemTag::SystemMaintenance => "SYSTEM_MAINTENANCE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Alert = 4,
    Security = 5,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Trace => "TRACE",
            Severity::Debug => "DEBUG",
            Severity::Info => "INFO",
            Severity::Warn => "WARN",
            Severity::Alert => "ALERT",
            Severity::Security => "SECURITY",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoProvenance {
    pub identity: String,
    pub token_prefix: String,
    pub session_id: String,
    pub peer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FromProvenance {
    pub source_device: String,
    pub thread_id: String,
    pub physical_addr: Option<String>,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToProvenance {
    pub destination_hardware: Option<String>,
    pub remote_relay: String,
    pub database_wal: String,
    pub client_ui: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatTelemetry {
    pub summary: String,
    pub rms_db: Option<f32>,
    pub baseline_db: Option<f32>,
    pub delta_db: Option<f32>,
    pub frames_captured: usize,
    pub shutter_latency_ns: u64,
    pub shutter_latency_ps: u128,
    pub payload_bytes: usize,
    pub payload_sha256: String,
    pub duration_ns: u64,
    pub duration_ps: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HowProvenance {
    pub protocol: String,
    pub transport: String,
    pub cipher: String,
    pub compression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinutiaeMetadata {
    pub cpu_rss_mb: f32,
    pub dsp_ema_alpha: f32,
    pub wal_page_count: u32,
    pub sqlite_commit_ns: u64,
    pub network_rtt_ms: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRecord {
    pub sequence: u64,
    pub timestamp_utc: String,
    pub timestamp_unix_ns: u128,
    pub node_id: String,
    pub subsystem: SubsystemTag,
    pub severity: Severity,
    pub who: WhoProvenance,
    pub from: FromProvenance,
    pub to: ToProvenance,
    pub what: WhatTelemetry,
    pub how: HowProvenance,
    pub minutiae: MinutiaeMetadata,
    pub prev_record_hash: String,
    pub record_hash: String,
}

impl TelemetryRecord {
    pub fn new(
        sequence: u64,
        node_id: &str,
        subsystem: SubsystemTag,
        severity: Severity,
        who: WhoProvenance,
        from: FromProvenance,
        to: ToProvenance,
        what: WhatTelemetry,
        how: HowProvenance,
        minutiae: MinutiaeMetadata,
        prev_record_hash: &str,
    ) -> Self {
        let nano = NanoTimestamp::now();
        let timestamp_utc = nano.to_rfc3339_nanos();
        let timestamp_unix_ns = nano.unix_nanos;

        let sanitized_summary = LogSanitizer::sanitize(&what.summary);
        let mut what = what;
        what.summary = sanitized_summary;

        let record_hash = Self::calculate_hash(
            sequence,
            timestamp_unix_ns,
            node_id,
            subsystem,
            severity,
            &what.summary,
            &what.payload_sha256,
            what.duration_ns,
            prev_record_hash,
        );

        Self {
            sequence,
            timestamp_utc,
            timestamp_unix_ns,
            node_id: node_id.to_string(),
            subsystem,
            severity,
            who,
            from,
            to,
            what,
            how,
            minutiae,
            prev_record_hash: prev_record_hash.to_string(),
            record_hash,
        }
    }

    pub fn calculate_hash(
        sequence: u64,
        timestamp_unix_ns: u128,
        node_id: &str,
        subsystem: SubsystemTag,
        severity: Severity,
        summary: &str,
        payload_sha256: &str,
        duration_ns: u64,
        prev_record_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&sequence.to_be_bytes());
        hasher.update(&timestamp_unix_ns.to_be_bytes());
        hasher.update(node_id.as_bytes());
        hasher.update(subsystem.as_str().as_bytes());
        hasher.update(severity.as_str().as_bytes());
        hasher.update(summary.as_bytes());
        hasher.update(payload_sha256.as_bytes());
        hasher.update(&duration_ns.to_be_bytes());
        hasher.update(prev_record_hash.as_bytes());

        hex::encode(hasher.finalize())
    }

    pub fn verify_integrity(&self) -> bool {
        let expected = Self::calculate_hash(
            self.sequence,
            self.timestamp_unix_ns,
            &self.node_id,
            self.subsystem,
            self.severity,
            &self.what.summary,
            &self.what.payload_sha256,
            self.what.duration_ns,
            &self.prev_record_hash,
        );
        self.record_hash == expected
    }

    pub fn verify_chain(records: &[TelemetryRecord]) -> Result<bool> {
        if records.is_empty() {
            return Ok(true);
        }

        let mut expected_prev_hash = "GENESIS_BLOCK_0000000000000000000000000000000000000000000000000000000000000000".to_string();

        for (idx, record) in records.iter().enumerate() {
            if idx > 0 && record.prev_record_hash != expected_prev_hash {
                return Err(TelemetryError::HashChainBroken {
                    sequence: record.sequence,
                    expected: expected_prev_hash,
                    actual: record.prev_record_hash.clone(),
                });
            }

            if !record.verify_integrity() {
                return Err(TelemetryError::HashChainBroken {
                    sequence: record.sequence,
                    expected: "VALID_INTEGRITY_HASH".into(),
                    actual: record.record_hash.clone(),
                });
            }

            expected_prev_hash = record.record_hash.clone();
        }

        Ok(true)
    }
}
