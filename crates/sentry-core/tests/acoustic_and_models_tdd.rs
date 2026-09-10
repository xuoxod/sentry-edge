//! TDD Test: Acoustic RMS Analysis & Alert Integrity

use sentry_core::{AcousticAnalyzer, IncidentSeverity, SentryAlert, SentryIncidentType};
use uuid::Uuid;

#[test]
fn test_acoustic_analyzer_quiet_vs_spike() {
    let mut analyzer = AcousticAnalyzer::new(35.0, 20.0, 48000);

    // 1. Quiet baseline noise buffer
    let quiet_samples = vec![150i16; 1024];
    let quiet_db = analyzer.compute_db_spl(&quiet_samples);
    assert!(quiet_db < 45.0, "Quiet samples should be low dB: {}", quiet_db);
    assert_eq!(analyzer.ingest_samples(&quiet_samples), None);

    // 2. Loud spike noise buffer
    let spike_samples = vec![25000i16; 1024];
    let spike_db = analyzer.compute_db_spl(&spike_samples);
    assert!(spike_db > 80.0, "Loud samples should be high dB: {}", spike_db);

    // Trigger should fire
    let trigger = analyzer.ingest_samples(&spike_samples);
    assert!(trigger.is_some(), "Spike trigger must fire on sudden volume jump");
}

#[test]
fn test_sentry_alert_sha256_integrity() {
    let node_id = Uuid::new_v4();
    let alert = SentryAlert::new(
        node_id,
        "Sentinel-Alpha",
        IncidentSeverity::High,
        SentryIncidentType::AcousticSpike {
            peak_db: 88.5,
            baseline_db: 32.0,
            delta_db: 56.5,
        },
        "Acoustic threshold breached (+56.5 dB)",
        Some("base64_jpeg_frame".to_string()),
    );

    assert_eq!(alert.node_label, "Sentinel-Alpha");
    assert_eq!(alert.severity, IncidentSeverity::High);
    assert_eq!(alert.event_signature_sha256.len(), 64);
    assert!(alert.verify_signature());
}
