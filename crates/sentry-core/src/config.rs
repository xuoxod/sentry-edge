//! Sentry Configuration

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryConfig {
    pub relay_url: String,
    pub auth_token: Option<String>,
    pub node_label: String,
    pub camera_device: String,
    pub audio_device: String,
    pub acoustic_trigger_delta_db: f32,
    pub snapshot_burst_count: usize,
    pub db_path: String,
}

impl Default for SentryConfig {
    fn default() -> Self {
        Self {
            relay_url: "ws://127.0.0.1:8084/ws/outpost".to_string(),
            auth_token: None,
            node_label: "Sentry Sentinel (hyperion-prime)".to_string(),
            camera_device: "/dev/video0".to_string(),
            audio_device: "default".to_string(),
            acoustic_trigger_delta_db: 20.0,
            snapshot_burst_count: 3,
            db_path: "./data/sentry_ledger.db".to_string(),
        }
    }
}
