//! TDD Chaos & Simulation Tests: 72-Hour Soak Endurance Report Generation

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
fn test_sim_72_hour_soak_report_synthesis() {
    let mut records = Vec::with_capacity(1000);
    let mut prev_hash = "GENESIS_BLOCK_0000000000000000000000000000000000000000000000000000000000000000".to_string();

    for i in 1..=1000 {
        let is_spike = i % 100 == 0;
        let rms = if is_spike { 88.5 } else { 50.0 + ((i % 15) as f32) * 0.8 };
        let base = 52.0;
        let delta = rms - base;

        let summary = if is_spike {
            format!("Transient acoustic spike breach: {:.1} dB SPL", rms)
        } else {
            format!("Periodic ambient baseline #{}: {:.1} dB SPL", i, rms)
        };

        let rec = test_record(
            i,
            "crunchbang-xua",
            if is_spike { SubsystemTag::CameraV4l2 } else { SubsystemTag::AudioDsp },
            if is_spike { Severity::Alert } else { Severity::Info },
            &summary,
            Some(rms),
            Some(base),
            Some(delta),
            &prev_hash,
        );
        prev_hash = rec.record_hash.clone();
        records.push(rec);
    }

    let doc = ReportOrchestrator::from_records("crunchbang-xua", "linux-x86_64", &records);

    assert_eq!(doc.metadata.total_records, 1000);
    assert_eq!(doc.metadata.sequence_start, 1);
    assert_eq!(doc.metadata.sequence_end, 1000);
    assert!(doc.metadata.is_chain_valid);
    assert_eq!(doc.summary.total_incidents, 10);
    assert!(doc.summary.narrative_story.contains("10 acoustic threshold breach events"));

    // Render HTML
    let html = ReportOrchestrator::render(&doc, ReportFormat::Html).unwrap();
    assert!(html.contains("1000 Verified Events"));
    assert!(html.contains("crunchbang-xua"));

    // Render Text
    let txt = ReportOrchestrator::render(&doc, ReportFormat::Text).unwrap();
    assert!(txt.contains("EXECUTIVE SECURITY & TELEMETRY DOSSIER"));
}
