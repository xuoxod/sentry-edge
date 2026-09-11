//! Sentry Report Single Source of Truth (SST) Domain Models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported multi-format export types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Html,
    Json,
    Jsonl,
    Csv,
    Text,
    Markdown,
}

impl std::str::FromStr for ReportFormat {
    type Err = crate::error::ReportError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "html" | "htm" => Ok(ReportFormat::Html),
            "json" => Ok(ReportFormat::Json),
            "jsonl" | "ndjson" => Ok(ReportFormat::Jsonl),
            "csv" => Ok(ReportFormat::Csv),
            "txt" | "text" | "plain" => Ok(ReportFormat::Text),
            "md" | "markdown" => Ok(ReportFormat::Markdown),
            other => Err(crate::error::ReportError::InvalidFormat(other.to_string())),
        }
    }
}

/// High-level executive threat and status assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutiveAssessment {
    /// Normal operations with nominal ambient sound levels
    Nominal,
    /// Elevated background noise or minor fluctuations
    Elevated,
    /// Acoustic threshold breaches or security events detected
    Alert,
    /// Cryptographic chain failure or sensor disconnection
    Critical,
}

impl ExecutiveAssessment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Nominal => "NOMINAL",
            Self::Elevated => "ELEVATED",
            Self::Alert => "ALERT_TRIGGERED",
            Self::Critical => "CRITICAL_ATTENTION",
        }
    }

    pub fn color_code(&self) -> &'static str {
        match self {
            Self::Nominal => "#238636", // Green
            Self::Elevated => "#d29922", // Amber
            Self::Alert => "#f85149",    // Red
            Self::Critical => "#da3633", // Dark Red
        }
    }
}

/// Metadata describing the sentinel node, environment, and verification status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub report_id: String,
    pub generated_at_utc: String,
    pub node_id: String,
    pub platform: String,
    pub session_start_utc: String,
    pub session_end_utc: String,
    pub elapsed_human: String,
    pub total_records: u64,
    pub sequence_start: u64,
    pub sequence_end: u64,
    pub genesis_hash: String,
    pub terminal_hash: String,
    pub is_chain_valid: bool,
    pub verification_notes: String,
}

/// Executive storytelling narrative and takeaways.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub assessment: ExecutiveAssessment,
    pub narrative_story: String,
    pub key_takeaways: Vec<String>,
    pub total_incidents: u64,
    pub total_camera_bursts: u64,
    pub total_bridge_dispatches: u64,
}

/// Quantitative acoustic insights and distribution metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcousticInsights {
    pub avg_baseline_db: f32,
    pub min_ambient_db: f32,
    pub max_peak_db: f32,
    pub max_delta_db: f32,
    pub spike_count: u64,
    pub quiet_period_pct: f32,
    pub moderate_period_pct: f32,
    pub loud_period_pct: f32,
}

/// Normalized canonical forensic event item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForensicEvent {
    pub sequence: u64,
    pub timestamp_utc: String,
    pub timestamp_unix_ns: String,
    pub subsystem: String,
    pub severity: String,
    pub summary: String,
    pub rms_db: Option<f32>,
    pub baseline_db: Option<f32>,
    pub delta_db: Option<f32>,
    pub duration_ns: u64,
    pub duration_ps: String,
    pub record_hash: String,
    pub prev_hash: String,

    // Rich 5W1H Micro-Provenance
    pub who_identity: String,
    pub who_session_id: String,
    pub who_token_prefix: String,
    pub who_peer_id: Option<String>,

    pub from_device: String,
    pub from_thread: String,
    pub from_endpoint: String,
    pub from_physical_addr: Option<String>,

    pub to_destination: Option<String>,
    pub to_relay: String,
    pub to_wal: String,
    pub to_client: Option<String>,

    pub how_protocol: String,
    pub how_transport: String,
    pub how_cipher: String,
    pub how_compression: Option<String>,

    // Optical Burst & Telemetry Payload Minutiae
    pub frames_captured: usize,
    pub shutter_latency_ns: u64,
    pub shutter_latency_ps: String,
    pub payload_bytes: usize,
    pub payload_sha256: String,

    // Engine Minutiae & Resource Footprint
    pub cpu_rss_mb: f32,
    pub dsp_ema_alpha: f32,
    pub wal_page_count: u32,
    pub sqlite_commit_ns: u64,
    pub network_rtt_ms: f32,
}

/// The Single Source of Truth (SST) Report Document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDocument {
    pub metadata: ReportMetadata,
    pub summary: ExecutiveSummary,
    pub acoustics: AcousticInsights,
    pub subsystem_breakdown: HashMap<String, u64>,
    pub severity_breakdown: HashMap<String, u64>,
    pub timeline: Vec<ForensicEvent>,
}
