//! TDD Test: Sentry Conduit Bridge & Alert Multiplexing

use sentry_bridge::SentryConduitBridge;
use sentry_core::{IncidentSeverity, SentryAlert, SentryIncidentType};
use uuid::Uuid;

#[test]
fn test_bridge_alert_framing_and_dispatch() {
    let node_id = Uuid::new_v4();
    let mut bridge = SentryConduitBridge::new(
        "ws://127.0.0.1:8084/ws/outpost",
        Some("test-secret-token".to_string()),
        node_id,
        "Sentinel-Alpha",
    );

    let alert = SentryAlert::new(
        node_id,
        "Sentinel-Alpha",
        IncidentSeverity::Critical,
        SentryIncidentType::AcousticSpike {
            peak_db: 94.2,
            baseline_db: 35.0,
            delta_db: 59.2,
        },
        "Acoustic threshold breach detected",
        None,
    );

    let packet = bridge.dispatch_alert(alert).unwrap();
    assert_eq!(packet.stream_id, 1);
    assert_eq!(packet.seq_num, 1);
    assert!(!packet.payload_json.is_empty());
    assert_eq!(bridge.queued_alerts.len(), 1);
}
