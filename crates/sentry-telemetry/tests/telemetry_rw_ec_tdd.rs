use rusqlite::Connection;
use sentry_telemetry::{
    FromProvenance, HowProvenance, MinutiaeMetadata, NanoTimestamp, PicoTimer,
    Severity, SubsystemTag, TelemetryEngine, TelemetryRecord, ToProvenance, WhatTelemetry,
    WhoProvenance,
};
use std::fs;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

fn dummy_provenance(summary: &str) -> (WhoProvenance, FromProvenance, ToProvenance, WhatTelemetry, HowProvenance, MinutiaeMetadata) {
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
        rms_db: Some(58.2),
        baseline_db: Some(40.1),
        delta_db: Some(18.1),
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
    (who, from, to, what, how, minutiae)
}

#[test]
fn test_rw_nanosecond_precision_and_provenance() {
    let timer = PicoTimer::start();
    let nano = NanoTimestamp::now();
    assert!(nano.unix_nanos > 1_700_000_000_000_000_000);
    assert!(nano.to_rfc3339_nanos().contains("T"));

    let elapsed_ns = timer.elapsed_nanos();
    let elapsed_ps = timer.elapsed_picos();
    assert!(elapsed_ps >= elapsed_ns as u128);

    let (who, from, to, what, how, minutiae) = dummy_provenance("Acoustic baseline updated");
    let record = TelemetryRecord::new(
        1,
        "crunchbang-laptop",
        SubsystemTag::AudioDsp,
        Severity::Info,
        who,
        from,
        to,
        what,
        how,
        minutiae,
        "GENESIS_HASH",
    );

    assert_eq!(record.sequence, 1);
    assert_eq!(record.node_id, "crunchbang-laptop");
    assert!(record.verify_integrity());

    let json = serde_json::to_string(&record).expect("Should serialize cleanly");
    let deserialized: TelemetryRecord = serde_json::from_str(&json).expect("Should deserialize cleanly");
    assert_eq!(deserialized.sequence, record.sequence);
    assert_eq!(deserialized.record_hash, record.record_hash);
}

#[test]
fn test_rw_file_sink_jsonl_append_and_readback() {
    let tmp = NamedTempFile::new().unwrap();
    let engine = TelemetryEngine::new("crunchbang-laptop", false)
        .with_file_sink(tmp.path())
        .expect("Should initialize file sink");

    for i in 1..=5 {
        let (who, from, to, what, how, minutiae) = dummy_provenance(&format!("Event #{}", i));
        engine.log(
            SubsystemTag::SystemHeartbeat,
            Severity::Debug,
            who,
            from,
            to,
            what,
            how,
            minutiae,
        ).unwrap();
    }

    assert_eq!(engine.current_sequence(), 5);

    let content = fs::read_to_string(tmp.path()).unwrap();
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 5);

    let first_record: TelemetryRecord = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(first_record.sequence, 1);
    assert_eq!(first_record.what.summary, "Event #1");
}

#[test]
fn test_rw_ledger_sink_sqlite_query() {
    let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));
    let engine = TelemetryEngine::new("crunchbang-laptop", false)
        .with_ledger_sink(conn.clone())
        .expect("Should initialize ledger sink");

    let (who, from, to, what, how, minutiae) = dummy_provenance("SQLite persistence test");
    engine.log(
        SubsystemTag::LedgerSqlite,
        Severity::Info,
        who,
        from,
        to,
        what,
        how,
        minutiae,
    ).unwrap();

    let conn_guard = conn.lock().unwrap();
    let mut stmt = conn_guard.prepare("SELECT sequence, summary, subsystem FROM telemetry_audit_ledger WHERE sequence = 1").unwrap();
    let row = stmt.query_row([], |r| {
        let seq: i64 = r.get(0)?;
        let summary: String = r.get(1)?;
        let sub: String = r.get(2)?;
        Ok((seq, summary, sub))
    }).unwrap();

    assert_eq!(row.0, 1);
    assert_eq!(row.1, "SQLite persistence test");
    assert_eq!(row.2, "LEDGER_SQLITE");
}

#[test]
fn test_ec_empty_fields_and_zero_duration() {
    let who = WhoProvenance {
        identity: "".into(),
        token_prefix: "".into(),
        session_id: "".into(),
        peer_id: None,
    };
    let from = FromProvenance {
        source_device: "".into(),
        thread_id: "".into(),
        physical_addr: None,
        endpoint: "".into(),
    };
    let to = ToProvenance {
        destination_hardware: None,
        remote_relay: "".into(),
        database_wal: "".into(),
        client_ui: None,
    };
    let what = WhatTelemetry {
        summary: "".into(),
        rms_db: None,
        baseline_db: None,
        delta_db: None,
        frames_captured: 0,
        shutter_latency_ns: 0,
        shutter_latency_ps: 0,
        payload_bytes: 0,
        payload_sha256: "".into(),
        duration_ns: 0,
        duration_ps: 0,
    };
    let how = HowProvenance {
        protocol: "".into(),
        transport: "".into(),
        cipher: "".into(),
        compression: None,
    };
    let minutiae = MinutiaeMetadata {
        cpu_rss_mb: 0.0,
        dsp_ema_alpha: 0.0,
        wal_page_count: 0,
        sqlite_commit_ns: 0,
        network_rtt_ms: 0.0,
    };

    let record = TelemetryRecord::new(
        1,
        "node",
        SubsystemTag::SystemHeartbeat,
        Severity::Trace,
        who,
        from,
        to,
        what,
        how,
        minutiae,
        "GENESIS",
    );

    assert!(record.verify_integrity());
}

#[test]
fn test_ec_extreme_volume_rapid_logging() {
    let engine = TelemetryEngine::new("crunchbang-laptop", false);
    for i in 1..=500 {
        let (who, from, to, what, how, minutiae) = dummy_provenance(&format!("Rapid log {}", i));
        engine.log(
            SubsystemTag::AudioDsp,
            Severity::Trace,
            who,
            from,
            to,
            what,
            how,
            minutiae,
        ).unwrap();
    }
    assert_eq!(engine.current_sequence(), 500);
}
