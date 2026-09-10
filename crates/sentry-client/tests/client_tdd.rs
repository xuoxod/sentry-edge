//! TDD Test: Sentry End-User Client & Alert Viewer

use sentry_client::SentryViewer;
use sentry_core::{IncidentSeverity, SentryAlert, SentryIncidentType};
use uuid::Uuid;

#[test]
fn test_sentry_viewer_ingest_alert() {
    let mut viewer = SentryViewer::new(
        "wss://relay.example.com:8084",
        Some("auth-token".to_string()),
        "operator-rick",
    );

    viewer.watch_node("Server-Room-Sentinel (hyperion-prime)");
    assert_eq!(viewer.monitored_nodes.len(), 1);

    let alert = SentryAlert::new(
        Uuid::new_v4(),
        "Server-Room-Sentinel (hyperion-prime)",
        IncidentSeverity::Critical,
        SentryIncidentType::AcousticSpike {
            peak_db: 96.0,
            baseline_db: 35.0,
            delta_db: 61.0,
        },
        "Loud bang / glass break detected in server room",
        None,
    );

    let banner = viewer.ingest_incoming_alert(alert);
    assert!(banner.contains("CRITICAL"));
    assert!(banner.contains("Server-Room-Sentinel"));
    assert_eq!(viewer.received_alerts.len(), 1);
}

#[test]
fn test_sentry_viewer_telepresence_connect() {
    let viewer = SentryViewer::new(
        "wss://relay.example.com:8084",
        None,
        "operator-rick",
    );

    let session = viewer.connect_telepresence(
        "https://sfu.example.com:7880",
        "hyperion-prime",
    ).unwrap();

    assert!(session.room_name.contains("hyperion-prime"));
    assert!(!session.token.is_empty());
}
