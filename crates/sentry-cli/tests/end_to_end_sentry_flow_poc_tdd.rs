//! TDD Test: End-to-End Sovereign Telepresence Sentinel Integration (POC Suite)

use sentry_bridge::SentryConduitBridge;
use sentry_client::SentryViewer;
use sentry_core::{
    config::SentryConfig, AcousticAnalyzer, IncidentSeverity, SentryAlert, SentryIncidentType,
};
use sentry_hardware::{SentryAudioSentinel, SentryCameraSentinel};
use sentry_ledger::SentryLedgerDb;
use sentry_report::{ReportFormat, ReportOrchestrator};
use uuid::Uuid;

#[test]
fn test_poc_end_to_end_acoustic_breach_to_client_delivery() {
    // 1. Initialize Sentry Configuration
    let config = SentryConfig::default();
    assert!(config.validate().is_ok());

    let node_id = Uuid::new_v4();
    let node_label = "Server-Room-Sentinel (hyperion-prime)";

    // 2. Initialize Hardware & Analytics Engines
    let mut analyzer = AcousticAnalyzer::new(
        config.hardware.acoustic_baseline_db,
        config.hardware.acoustic_trigger_delta_db,
        48000,
    );
    let camera = SentryCameraSentinel::new(&config.hardware.camera_device);
    let audio = SentryAudioSentinel::new();
    let ledger = SentryLedgerDb::new_in_memory().expect("SQLite in-memory DB must initialize");
    let mut bridge = SentryConduitBridge::new(
        &config.network.relay_url,
        config.network.auth_token.clone(),
        node_id,
        node_label,
    );

    // 3. Audio Watchdog runs on ambient room sound to establish baseline
    let ambient_db = audio.sample_ambient_db().unwrap_or(35.0);
    assert!(ambient_db > 0.0);
    let normal_ambient = vec![200i16; 1024];
    assert_eq!(analyzer.ingest_samples(&normal_ambient), None);

    // Now sudden acoustic breach occurs (+55 dB sound spike)
    let loud_acoustic_burst = vec![28000i16; 1024];
    let breach_peak = analyzer.ingest_samples(&loud_acoustic_burst);
    assert!(breach_peak.is_some(), "Spike must be detected by acoustic analyzer");
    let peak_db = breach_peak.unwrap();

    // 4. Hardware Sentinel triggers V4L2 5-frame burst
    let burst_frames = camera
        .capture_burst(config.hardware.snapshot_burst_count)
        .expect("Camera burst must capture frames");
    assert_eq!(burst_frames.len(), 5);

    // 5. Generate and sign SentryAlert
    let alert = SentryAlert::new(
        node_id,
        node_label,
        IncidentSeverity::Critical,
        SentryIncidentType::AcousticSpike {
            peak_db,
            baseline_db: analyzer.baseline_db,
            delta_db: peak_db - analyzer.baseline_db,
        },
        "Acoustic threshold breach in server room (+55.0 dB)",
        Some(format!("jpeg_base64_payload_len_{}", burst_frames[0].len())),
    );

    assert!(alert.verify_signature(), "Alert signature must be valid");

    // 6. Dispatch over Conduit WSS Bridge
    let wire_packet = bridge.dispatch_alert(alert.clone()).expect("Must frame wire packet");
    assert_eq!(wire_packet.stream_id, 1);
    assert_eq!(wire_packet.seq_num, 1);

    // 7. Persist to SQLite WAL Ledger
    ledger.insert_alert(&alert).expect("Must persist incident to SQLite");
    assert_eq!(ledger.count_alerts().unwrap(), 1);

    // 8. Generate Multi-Format HTML Security Dossier
    let doc = ReportOrchestrator::from_records(node_label, "linux-x86_64", &[]);
    let report_html = ReportOrchestrator::render(&doc, ReportFormat::Html).unwrap();
    assert!(report_html.contains("SENTRY-EDGE"));
    assert!(report_html.contains("hyperion-prime"));

    // 9. Remote End-User Client ingests alert
    let mut client = SentryViewer::new(
        &config.network.relay_url,
        config.network.auth_token,
        "operator-rick",
    );
    client.watch_node(node_label);
    let banner = client.ingest_incoming_alert(alert);

    assert!(banner.contains("CRITICAL"));
    assert!(banner.contains("hyperion-prime"));
    assert_eq!(client.received_alerts.len(), 1);
}
