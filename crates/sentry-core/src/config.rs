//! Sentry Sovereign Configuration Loader & Generator

use crate::error::{SentryError, SentryResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main persistent configuration structure for Sentry-Edge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SentryConfig {
    pub node: NodeSection,
    pub network: NetworkSection,
    pub hardware: HardwareSection,
    pub telepresence: TelepresenceSection,
    pub storage: StorageSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeSection {
    pub label: String,
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkSection {
    pub relay_url: String,
    pub auth_token: Option<String>,
    pub connect_timeout_secs: u64,
    pub heartbeat_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HardwareSection {
    pub camera_device: String,
    pub audio_device: String,
    pub acoustic_baseline_db: f32,
    pub acoustic_trigger_delta_db: f32,
    pub snapshot_burst_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelepresenceSection {
    pub sfu_url: String,
    pub sfu_api_key: Option<String>,
    pub sfu_api_secret: Option<String>,
    pub default_room: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageSection {
    pub ledger_db_path: String,
    pub export_dir: String,
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            node: NodeSection {
                label: "Server-Room-Sentinel (hyperion-prime)".to_string(),
                environment: "production".to_string(),
            },
            network: NetworkSection {
                relay_url: "wss://relay.example.com:8084/ws/outpost".to_string(),
                auth_token: None,
                connect_timeout_secs: 10,
                heartbeat_interval_secs: 30,
            },
            hardware: HardwareSection {
                camera_device: "/dev/video0".to_string(),
                audio_device: "default".to_string(),
                acoustic_baseline_db: 35.0,
                acoustic_trigger_delta_db: 20.0,
                snapshot_burst_count: 5,
            },
            telepresence: TelepresenceSection {
                sfu_url: "https://sfu.example.com:7880".to_string(),
                sfu_api_key: None,
                sfu_api_secret: None,
                default_room: "sentry-telepresence-main".to_string(),
            },
            storage: StorageSection {
                ledger_db_path: "./data/sentry_ledger.db".to_string(),
                export_dir: "./reports".to_string(),
            },
        }
    }
}

impl SentryConfig {
    /// Generate a well-commented TOML configuration template.
    pub fn default_template() -> String {
        r#"# ==============================================================================
# SENTRY-EDGE // Sovereign Telepresence Sentinel Configuration
# Default Location: ~/.config/sentry/sentry.toml
# ==============================================================================

[node]
label = "Server-Room-Sentinel (hyperion-prime)"
environment = "production"

[network]
# Target self-hosted Conduit Relay WebSocket gateway
relay_url = "wss://relay.example.com:8084/ws/outpost"
# Optional secret master pairing token
# auth_token = "your-secret-conduit-token"
connect_timeout_secs = 10
heartbeat_interval_secs = 30

[hardware]
# V4L2 camera capture device
camera_device = "/dev/video0"
# ALSA / PipeWire sound input
audio_device = "default"
# Baseline ambient noise decibels (dB SPL)
acoustic_baseline_db = 35.0
# Sudden noise jump threshold (+dB above baseline to trigger alert)
acoustic_trigger_delta_db = 20.0
# Multi-frame camera snapshot burst count upon breach
snapshot_burst_count = 5

[telepresence]
# Optional self-hosted LiveKit SFU media server for sub-10ms WebRTC
sfu_url = "https://sfu.example.com:7880"
# sfu_api_key = "your-livekit-api-key"
# sfu_api_secret = "your-livekit-api-secret"
default_room = "sentry-telepresence-main"

[storage]
ledger_db_path = "./data/sentry_ledger.db"
export_dir = "./reports"
"#
        .to_string()
    }

    /// Load configuration from a TOML string with validation.
    pub fn from_toml_str(toml_str: &str) -> SentryResult<Self> {
        let config: SentryConfig = toml::from_str(toml_str)
            .map_err(|e| SentryError::Config(format!("Malformed TOML syntax: {}", e)))?;
        config.validate()?;
        Ok(config)
    }

    /// Load configuration from file path.
    pub fn load_from_file(path: &Path) -> SentryResult<Self> {
        if !path.exists() {
            return Err(SentryError::Config(format!(
                "Config file not found at: {}",
                path.display()
            )));
        }
        let content = std::fs::read_to_string(path)?;
        Self::from_toml_str(&content)
    }

    /// Discover configuration automatically in order of precedence:
    /// 1. Path in `SENTRY_CONFIG` env var
    /// 2. OS Standard config path (`~/.config/sentry/sentry.toml`, `%APPDATA%\sentry\sentry.toml`, `~/Library/Application Support/sentry/sentry.toml`)
    /// 3. `/etc/sentry/sentry.toml` (Unix systems)
    /// 4. `./sentry.toml`
    /// 5. In-memory defaults
    pub fn discover() -> Self {
        if let Ok(path_str) = std::env::var("SENTRY_CONFIG") {
            if let Ok(cfg) = Self::load_from_file(Path::new(&path_str)) {
                return cfg;
            }
        }

        let user_cfg = crate::platform::PlatformPaths::default_config_path();
        if let Ok(cfg) = Self::load_from_file(&user_cfg) {
            return cfg;
        }

        let etc_cfg = Path::new("/etc/sentry/sentry.toml");
        if let Ok(cfg) = Self::load_from_file(etc_cfg) {
            return cfg;
        }

        let local_cfg = Path::new("./sentry.toml");
        if let Ok(cfg) = Self::load_from_file(local_cfg) {
            return cfg;
        }

        Self::default()
    }

    /// Defensive validation and security sanitization checks.
    pub fn validate(&self) -> SentryResult<()> {
        if self.node.label.trim().is_empty() {
            return Err(SentryError::Config("Node label cannot be empty".to_string()));
        }

        if !self.network.relay_url.starts_with("ws://") && !self.network.relay_url.starts_with("wss://") {
            return Err(SentryError::Config(format!(
                "Invalid relay URL '{}'. Must start with ws:// or wss://",
                self.network.relay_url
            )));
        }

        if self.hardware.acoustic_trigger_delta_db <= 0.0 || self.hardware.acoustic_trigger_delta_db > 100.0 {
            return Err(SentryError::Config(format!(
                "Acoustic trigger delta ({:.1} dB) out of sane bounds (0.1 .. 100.0)",
                self.hardware.acoustic_trigger_delta_db
            )));
        }

        if self.hardware.snapshot_burst_count == 0 || self.hardware.snapshot_burst_count > 50 {
            return Err(SentryError::Config(format!(
                "Snapshot burst count ({}) out of safe range (1 .. 50)",
                self.hardware.snapshot_burst_count
            )));
        }

        // Defense against path traversal attacks in database and export paths
        crate::platform::PlatformPaths::validate_safe_path(Path::new(&self.storage.ledger_db_path))?;
        crate::platform::PlatformPaths::validate_safe_path(Path::new(&self.storage.export_dir))?;

        Ok(())
    }
}

