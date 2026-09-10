use rusqlite::Connection;
use sentry_telemetry::{
    FromProvenance, HowProvenance, MinutiaeMetadata, Severity, SubsystemTag,
    TelemetryEngine, TelemetryRecord, ToProvenance, WhatTelemetry, WhoProvenance,
};
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;

#[test]
fn test_sim_72_hour_soak_event_generator() {
    let tmp = NamedTempFile::new().unwrap();
    let conn = Arc::new(Mutex::new(Connection::open_in_memory().unwrap()));

    let engine = TelemetryEngine::new("crunchbang-laptop", false)
        .with_file_sink(tmp.path())
        .unwrap()
        .with_ledger_sink(conn.clone())
        .unwrap();

    let mut records = Vec::new();
    let mut ambient_baseline = 38.0f32;

    // Simulate 72-hour timeline cycles:
    // Morning routine (4:20 AM), commute, daytime idle/HVAC drift, evening TV/chatter, nighttime silence
    for hour in 0..72 {
        let is_morning = (hour % 24) == 4;
        let is_night = (hour % 24) >= 22 || (hour % 24) <= 3;

        // Baseline drift
        if is_night {
            ambient_baseline = (ambient_baseline * 0.95 + 32.0 * 0.05).max(30.0);
        } else if is_morning {
            ambient_baseline = (ambient_baseline * 0.9 + 52.0 * 0.1).min(65.0);
        } else {
            ambient_baseline = ambient_baseline * 0.98 + 42.0 * 0.02;
        }

        let is_spike = is_morning || (hour % 17 == 0);
        let measured_rms = if is_spike { ambient_baseline + 22.5 } else { ambient_baseline + 1.2 };
        let delta = measured_rms - ambient_baseline;
        let severity = if is_spike { Severity::Alert } else { Severity::Debug };
        let subsystem = if is_spike { SubsystemTag::CameraV4l2 } else { SubsystemTag::AudioDsp };
        let summary = if is_spike {
            format!("Hour {}: Acoustic breach triggered frame burst (+{:.1}dB)", hour, delta)
        } else {
            format!("Hour {}: Ambient noise baseline nominal ({:.1}dB)", hour, ambient_baseline)
        };

        let who = WhoProvenance {
            identity: "sovereign-daemon".into(),
            token_prefix: "sentry-dev-99x".into(),
            session_id: format!("sess-hour-{}", hour),
            peer_id: None,
        };
        let from = FromProvenance {
            source_device: "alsa://hw:0,0".into(),
            thread_id: format!("worker-{}", hour % 4),
            physical_addr: Some("00:15:5d:aa:bb:cc".into()),
            endpoint: "ipc://dsp".into(),
        };
        let to = ToProvenance {
            destination_hardware: if is_spike { Some("/dev/video0".into()) } else { None },
            remote_relay: "wss://relay.example.com/ws/outpost".into(),
            database_wal: "sentry_ledger.db".into(),
            client_ui: Some("sentry-viewer-app".into()),
        };
        let what = WhatTelemetry {
            summary,
            rms_db: Some(measured_rms),
            baseline_db: Some(ambient_baseline),
            delta_db: Some(delta),
            frames_captured: if is_spike { 5 } else { 0 },
            shutter_latency_ns: if is_spike { 1_200_000 } else { 0 },
            shutter_latency_ps: if is_spike { 1_200_000_000 } else { 0 },
            payload_bytes: if is_spike { 8192 } else { 64 },
            payload_sha256: format!("sha256-hour-{}", hour),
            duration_ns: 45_000,
            duration_ps: 45_000_000,
        };
        let how = HowProvenance {
            protocol: "ALSA_PCM -> SQLITE_WAL".into(),
            transport: "Local Hardware Loop".into(),
            cipher: "ChaCha20-Poly1305".into(),
            compression: None,
        };
        let minutiae = MinutiaeMetadata {
            cpu_rss_mb: 19.4 + (hour as f32 * 0.01),
            dsp_ema_alpha: 0.15,
            wal_page_count: 10 + (hour as u32),
            sqlite_commit_ns: 25_000,
            network_rtt_ms: 1.1,
        };

        let rec = engine.log(subsystem, severity, who, from, to, what, how, minutiae).unwrap();
        records.push(rec);
    }

    assert_eq!(records.len(), 72);
    assert!(TelemetryRecord::verify_chain(&records).is_ok());

    let conn_guard = conn.lock().unwrap();
    let count: i64 = conn_guard.query_row("SELECT count(*) FROM telemetry_audit_ledger", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 72);
}
