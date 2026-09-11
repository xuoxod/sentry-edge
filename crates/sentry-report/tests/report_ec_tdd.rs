//! TDD Edge-Case Tests for Sentry Report Engine

use sentry_report::{ReportFormat, ReportOrchestrator};
use sentry_telemetry::{
    FromProvenance, HowProvenance, MinutiaeMetadata, Severity, SubsystemTag, TelemetryRecord,
    ToProvenance, WhatTelemetry, WhoProvenance,
};

fn test_record(
    sequence: u64,
    node_id: &str,
    subsystem: SubsystemTag,
    severity: Severity,
    summary: &str,
    rms: Option<f32>,
    base: Option<f32>,
    delta: Option<f32>,
    prev_hash: &str,
) -> TelemetryRecord {
    let who = WhoProvenance {
        identity: "sentinel-core".to_string(),
        token_prefix: "tok-99x".to_string(),
        session_id: "sess-01".to_string(),
        peer_id: None,
    };
    let from = FromProvenance {
        source_device: "alsa://hw:0,0".to_string(),
        thread_id: "dsp-thread-1".to_string(),
        physical_addr: Some("b8:27:eb:11:22:33".to_string()),
        endpoint: "ipc://dsp-pipe".to_string(),
    };
    let to = ToProvenance {
        destination_hardware: Some("/dev/video0".to_string()),
        remote_relay: "wss://relay.example.com/ws/outpost".to_string(),
        database_wal: "sentry_ledger.db".to_string(),
        client_ui: Some("sentry-viewer-app".to_string()),
    };
    let what = WhatTelemetry {
        summary: summary.to_string(),
        rms_db: rms,
        baseline_db: base,
        delta_db: delta,
        frames_captured: 5,
        shutter_latency_ns: 125_000,
        shutter_latency_ps: 125_000_000,
        payload_bytes: 4096,
        payload_sha256: "aabbccdd11223344".to_string(),
        duration_ns: 250_000,
        duration_ps: 250_000_000,
    };
    let how = HowProvenance {
        protocol: "ALSA_PCM -> V4L2_MMAP -> WSS_TLS".to_string(),
        transport: "Local Hardware Loop -> TLS Socket".to_string(),
        cipher: "ChaCha20-Poly1305 / HMAC-SHA256".to_string(),
        compression: Some("zstd".to_string()),
    };
    let minutiae = MinutiaeMetadata {
        cpu_rss_mb: 18.2,
        dsp_ema_alpha: 0.15,
        wal_page_count: 14,
        sqlite_commit_ns: 42_000,
        network_rtt_ms: 1.25,
    };
    TelemetryRecord::new(
        sequence,
        node_id,
        subsystem,
        severity,
        who,
        from,
        to,
        what,
        how,
        minutiae,
        prev_hash,
    )
}

#[test]
fn test_ec_empty_records_dataset() {
    let doc = ReportOrchestrator::from_records("empty-node", "linux-x86_64", &[]);
    assert_eq!(doc.metadata.total_records, 0);
    assert_eq!(doc.timeline.len(), 0);

    let html = ReportOrchestrator::render(&doc, ReportFormat::Html).unwrap();
    assert!(html.contains("empty-node"));

    let csv = ReportOrchestrator::render(&doc, ReportFormat::Csv).unwrap();
    assert!(csv.contains("Timestamp_UTC"));

    let txt = ReportOrchestrator::render(&doc, ReportFormat::Text).unwrap();
    assert!(txt.contains("No telemetry records were found"));

    let json = ReportOrchestrator::render(&doc, ReportFormat::Json).unwrap();
    assert!(json.contains("\"total_records\": 0"));
}

#[test]
fn test_ec_single_record_dataset() {
    let rec = test_record(
        1,
        "lone-node",
        SubsystemTag::SystemHeartbeat,
        Severity::Info,
        "Sentinel initial startup",
        None,
        None,
        None,
        "GENESIS_BLOCK_0000000000000000000000000000000000000000000000000000000000000000",
    );

    let doc = ReportOrchestrator::from_records("lone-node", "linux-x86_64", &[rec]);
    assert_eq!(doc.metadata.total_records, 1);
    assert!(doc.metadata.is_chain_valid);

    let md = ReportOrchestrator::render(&doc, ReportFormat::Markdown).unwrap();
    assert!(md.contains("Sentinel initial startup"));
}

#[test]
fn test_ec_unicode_and_special_characters_in_summary() {
    let rec = test_record(
        1,
        "special-node-🚀",
        SubsystemTag::AudioDsp,
        Severity::Alert,
        "Sound alert: \"Glass Shatter!\" & <door slam>",
        Some(95.4),
        Some(40.0),
        Some(55.4),
        "GENESIS_BLOCK_0000000000000000000000000000000000000000000000000000000000000000",
    );

    let doc = ReportOrchestrator::from_records("special-node-🚀", "linux-x86_64", &[rec]);
    let html = ReportOrchestrator::render(&doc, ReportFormat::Html).unwrap();

    // Verify HTML escaping
    assert!(html.contains("&lt;door slam&gt;"));
    assert!(html.contains("&quot;Glass Shatter!&quot;"));

    let csv = ReportOrchestrator::render(&doc, ReportFormat::Csv).unwrap();
    assert!(csv.contains("\"Sound alert: \"\"Glass Shatter!\"\" & <door slam>\""));
}
