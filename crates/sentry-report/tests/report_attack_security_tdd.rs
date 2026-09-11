//! TDD Attack & Security Tests for Sentry Report Engine

use sentry_report::{ExecutiveAssessment, ReportFormat, ReportOrchestrator};
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
fn test_attack_csv_formula_injection_defense() {
    let rec = test_record(
        1,
        "node-victim",
        SubsystemTag::AudioDsp,
        Severity::Alert,
        "=cmd|'/C calc'!A0",
        Some(99.0),
        Some(40.0),
        Some(59.0),
        "GENESIS_BLOCK_0000000000000000000000000000000000000000000000000000000000000000",
    );

    let doc = ReportOrchestrator::from_records("node-victim", "linux-x86_64", &[rec]);
    let csv = ReportOrchestrator::render(&doc, ReportFormat::Csv).unwrap();

    // Verify cell is prefixed with ' to prevent Excel from executing formula
    assert!(csv.contains("'=cmd|'/C calc'!A0"));
}

#[test]
fn test_attack_html_xss_injection_escaping() {
    let rec = test_record(
        1,
        "node-<script>alert('pwned')</script>",
        SubsystemTag::AudioDsp,
        Severity::Alert,
        "<b onmouseover=alert('xss')>Evil Payload</b>",
        Some(85.0),
        Some(40.0),
        Some(45.0),
        "GENESIS_BLOCK_0000000000000000000000000000000000000000000000000000000000000000",
    );

    let doc = ReportOrchestrator::from_records("node-<script>alert('pwned')</script>", "linux-x86_64", &[rec]);
    let html = ReportOrchestrator::render(&doc, ReportFormat::Html).unwrap();

    // Ensure raw scripts/handlers do not exist in HTML
    assert!(!html.contains("<script>alert('pwned')</script>"));
    assert!(html.contains("&lt;script&gt;alert(&#39;pwned&#39;)&lt;/script&gt;"));
    assert!(html.contains("&lt;b onmouseover=alert(&#39;xss&#39;)&gt;Evil Payload&lt;/b&gt;"));
}

#[test]
fn test_attack_tampered_hash_chain_detection_in_report() {
    let rec1 = test_record(
        1,
        "secure-node",
        SubsystemTag::AudioDsp,
        Severity::Info,
        "Normal baseline 50dB",
        Some(50.0),
        Some(50.0),
        Some(0.0),
        "GENESIS_BLOCK_0000000000000000000000000000000000000000000000000000000000000000",
    );

    let mut rec2 = test_record(
        2,
        "secure-node",
        SubsystemTag::AudioDsp,
        Severity::Alert,
        "Intrusion detected",
        Some(95.0),
        Some(50.0),
        Some(45.0),
        &rec1.record_hash,
    );

    // Tamper with rec2 summary without updating hash
    rec2.what.summary = "TAMPERED: All is quiet".to_string();

    let doc = ReportOrchestrator::from_records("secure-node", "linux-x86_64", &[rec1, rec2]);

    assert_eq!(doc.summary.assessment, ExecutiveAssessment::Critical);
    assert!(!doc.metadata.is_chain_valid);
    assert!(doc.summary.narrative_story.contains("CRITICAL ALERT: Tampering or discontinuity was detected"));

    let html = ReportOrchestrator::render(&doc, ReportFormat::Html).unwrap();
    assert!(html.contains("TAMPER DETECTED / DISCONTINUOUS"));
}
