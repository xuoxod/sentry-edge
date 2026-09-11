//! TDD Real-World Tests for Sentry Report Engine Multi-Format Exporters

use sentry_report::{ReportFormat, ReportOrchestrator};
use sentry_telemetry::{
    FromProvenance, HowProvenance, MinutiaeMetadata, Severity, SubsystemTag, TelemetryRecord,
    ToProvenance, WhatTelemetry, WhoProvenance,
};
use tempfile::tempdir;

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

fn create_sample_records() -> Vec<TelemetryRecord> {
    let mut records = Vec::new();
    let mut prev_hash = "GENESIS_BLOCK_0000000000000000000000000000000000000000000000000000000000000000".to_string();

    for i in 1..=5 {
        let is_spike = i == 3;
        let summary = if is_spike {
            "Acoustic spike breach: 86.2 dB SPL (+33.7 dB over baseline)".to_string()
        } else {
            format!("Ambient baseline #{}: 52.0 dB SPL", i)
        };
        let rms = if is_spike { 86.2 } else { 52.0 + (i as f32) * 0.2 };
        let delta = if is_spike { 34.2 } else { 0.2 * (i as f32) };

        let rec = test_record(
            i,
            "node-alpha",
            if is_spike { SubsystemTag::CameraV4l2 } else { SubsystemTag::AudioDsp },
            if is_spike { Severity::Alert } else { Severity::Info },
            &summary,
            Some(rms),
            Some(52.0),
            Some(delta),
            &prev_hash,
        );
        prev_hash = rec.record_hash.clone();
        records.push(rec);
    }
    records
}

#[test]
fn test_rw_report_orchestrator_all_formats_roundtrip() {
    let records = create_sample_records();
    let doc = ReportOrchestrator::from_records("node-alpha", "linux-x86_64", &records);

    assert_eq!(doc.metadata.total_records, 5);
    assert_eq!(doc.metadata.sequence_start, 1);
    assert_eq!(doc.metadata.sequence_end, 5);
    assert!(doc.metadata.is_chain_valid);
    assert_eq!(doc.summary.total_incidents, 1);

    // 1. HTML
    let html = ReportOrchestrator::render(&doc, ReportFormat::Html).unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("SENTRY-EDGE // EXECUTIVE DOSSIER"));
    assert!(html.contains("Acoustic spike breach"));
    assert!(html.contains("SHA-256 HASH CHAIN VERIFIED"));

    // 2. JSON
    let json = ReportOrchestrator::render(&doc, ReportFormat::Json).unwrap();
    assert!(json.contains("\"node_id\": \"node-alpha\""));
    assert!(json.contains("\"total_records\": 5"));

    // 3. JSONL
    let jsonl = ReportOrchestrator::render(&doc, ReportFormat::Jsonl).unwrap();
    assert!(jsonl.contains("REPORT_MANIFEST"));
    assert_eq!(jsonl.lines().count(), 6); // 1 manifest + 5 events

    // 4. CSV
    let csv = ReportOrchestrator::render(&doc, ReportFormat::Csv).unwrap();
    assert!(csv.contains("Sequence,Timestamp_UTC"));
    assert!(csv.contains("86.2,52.0,34.2"));

    // 5. Text
    let txt = ReportOrchestrator::render(&doc, ReportFormat::Text).unwrap();
    assert!(txt.contains("EXECUTIVE SECURITY & TELEMETRY DOSSIER"));
    assert!(txt.contains("EXECUTIVE SUMMARY & SITUATION NARRATIVE"));
    assert!(txt.contains("CHRONOLOGICAL FORENSIC AUDIT TIMELINE"));

    // 6. Markdown
    let md = ReportOrchestrator::render(&doc, ReportFormat::Markdown).unwrap();
    assert!(md.contains("# 🛡️ SENTRY-EDGE Security & Telemetry Dossier"));
    assert!(md.contains("| **Monitored Node** | `node-alpha` |"));
}

#[test]
fn test_rw_report_render_to_file_and_infer() {
    let tmp = tempdir().unwrap();
    let records = create_sample_records();
    let doc = ReportOrchestrator::from_records("node-beta", "linux-x86_64", &records);

    let html_path = tmp.path().join("dossier.html");
    let json_path = tmp.path().join("dossier.json");
    let csv_path = tmp.path().join("dossier.csv");

    assert_eq!(ReportOrchestrator::infer_format(&html_path), ReportFormat::Html);
    assert_eq!(ReportOrchestrator::infer_format(&json_path), ReportFormat::Json);
    assert_eq!(ReportOrchestrator::infer_format(&csv_path), ReportFormat::Csv);

    ReportOrchestrator::render_to_file(&doc, ReportFormat::Html, &html_path).unwrap();
    ReportOrchestrator::render_to_file(&doc, ReportFormat::Json, &json_path).unwrap();
    ReportOrchestrator::render_to_file(&doc, ReportFormat::Csv, &csv_path).unwrap();

    assert!(html_path.exists());
    assert!(json_path.exists());
    assert!(csv_path.exists());

    let html_content = std::fs::read_to_string(&html_path).unwrap();
    assert!(html_content.contains("node-beta"));
}
