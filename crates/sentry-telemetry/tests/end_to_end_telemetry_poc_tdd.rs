use rusqlite::Connection;
use sentry_telemetry::{
    FromProvenance, HowProvenance, MinutiaeMetadata, PicoTimer, Severity,
    SubsystemTag, TelemetryEngine, TelemetryRecord, ToProvenance, WhatTelemetry, WhoProvenance,
};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

#[test]
fn test_poc_end_to_end_sentry_telemetry_lifecycle() {
    let tmp = NamedTempFile::new().unwrap();
    let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
    let timer = PicoTimer::start();

    // 1. Initialize Engine with FileSink & LedgerSink
    let engine = TelemetryEngine::new("crunchbang-laptop", false)
        .with_file_sink(tmp.path())
        .expect("File sink must initialize")
        .with_ledger_sink(conn.clone())
        .expect("Ledger sink must initialize");

    // 2. Log acoustic monitoring tick
    let who = WhoProvenance {
        identity: "sentinel-core".into(),
        token_prefix: "sentry-dev-99x".into(),
        session_id: "sess-lifecycle-01".into(),
        peer_id: None,
    };
    let from = FromProvenance {
        source_device: "alsa://hw:0,0".into(),
        thread_id: "dsp-main".into(),
        physical_addr: Some("00:15:5d:01:02:03".into()),
        endpoint: "ipc://dsp".into(),
    };
    let to = ToProvenance {
        destination_hardware: Some("/dev/video0".into()),
        remote_relay: "wss://relay.example.com/ws/outpost".into(),
        database_wal: "sentry_ledger.db".into(),
        client_ui: Some("sentry-viewer-app".into()),
    };
    let what = WhatTelemetry {
        summary: "Acoustic breach +21.4dB detected, initiating 5-frame JPEG capture".into(),
        rms_db: Some(65.4),
        baseline_db: Some(44.0),
        delta_db: Some(21.4),
        frames_captured: 5,
        shutter_latency_ns: 1_450_000,
        shutter_latency_ps: 1_450_000_000,
        payload_bytes: 8192,
        payload_sha256: "99aabbccddeeff00112233".into(),
        duration_ns: timer.elapsed_nanos(),
        duration_ps: timer.elapsed_picos(),
    };
    let how = HowProvenance {
        protocol: "ALSA_PCM -> V4L2_MMAP -> SQLITE_WAL -> WSS_TLS".into(),
        transport: "Local Hardware Loop -> TLS Socket".into(),
        cipher: "ChaCha20-Poly1305 / HMAC-SHA256".into(),
        compression: Some("zstd".into()),
    };
    let minutiae = MinutiaeMetadata {
        cpu_rss_mb: 20.1,
        dsp_ema_alpha: 0.15,
        wal_page_count: 32,
        sqlite_commit_ns: 35_000,
        network_rtt_ms: 0.95,
    };

    let record = engine.log(
        SubsystemTag::CameraV4l2,
        Severity::Alert,
        who,
        from,
        to,
        what,
        how,
        minutiae,
    ).expect("Log emission must succeed");

    // 3. Verify record integrity
    assert_eq!(record.sequence, 1);
    assert!(record.verify_integrity());
    assert!(!record.prev_record_hash.is_empty());
    assert!(!record.record_hash.is_empty());

    // 4. Verify chain integrity
    assert!(TelemetryRecord::verify_chain(&[record]).is_ok());

    // 5. Verify SQLite WAL persistence
    let conn_guard = conn.lock().unwrap();
    let row: (i64, String, String) = conn_guard.query_row(
        "SELECT sequence, summary, severity FROM telemetry_audit_ledger WHERE sequence = 1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).expect("Row must exist in SQLite ledger");

    assert_eq!(row.0, 1);
    assert!(row.1.contains("Acoustic breach"));
    assert_eq!(row.2, "ALERT");
}
