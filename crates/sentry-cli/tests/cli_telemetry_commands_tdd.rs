use sentry_telemetry::{
    FromProvenance, HowProvenance, MinutiaeMetadata, PicoTimer, Severity,
    SubsystemTag, TelemetryEngine, TelemetryRecord, ToProvenance, WhatTelemetry, WhoProvenance,
};

#[test]
fn test_cli_telemetry_hash_chain_and_stats() {
    let engine = TelemetryEngine::new("crunchbang-laptop", false);
    let mut records = Vec::new();

    for i in 1..=15 {
        let timer = PicoTimer::start();
        let who = WhoProvenance {
            identity: "daemon".into(),
            token_prefix: "sentry-dev-99x".into(),
            session_id: "sess-01".into(),
            peer_id: None,
        };
        let from = FromProvenance {
            source_device: "alsa".into(),
            thread_id: "t-1".into(),
            physical_addr: None,
            endpoint: "ipc".into(),
        };
        let to = ToProvenance {
            destination_hardware: None,
            remote_relay: "relay".into(),
            database_wal: "db".into(),
            client_ui: None,
        };
        let what = WhatTelemetry {
            summary: format!("Audit record #{}", i),
            rms_db: Some(40.0 + (i as f32 * 0.5)),
            baseline_db: Some(38.0),
            delta_db: Some(2.0 + (i as f32 * 0.5)),
            frames_captured: if i % 5 == 0 { 5 } else { 0 },
            shutter_latency_ns: if i % 5 == 0 { 1_000_000 } else { 0 },
            shutter_latency_ps: if i % 5 == 0 { 1_000_000_000 } else { 0 },
            payload_bytes: 128,
            payload_sha256: format!("sig-{}", i),
            duration_ns: timer.elapsed_nanos(),
            duration_ps: timer.elapsed_picos(),
        };
        let how = HowProvenance {
            protocol: "ALSA_PCM".into(),
            transport: "Loop".into(),
            cipher: "None".into(),
            compression: None,
        };
        let minutiae = MinutiaeMetadata {
            cpu_rss_mb: 18.0,
            dsp_ema_alpha: 0.15,
            wal_page_count: 5,
            sqlite_commit_ns: 20_000,
            network_rtt_ms: 1.0,
        };

        let rec = engine.log(
            if i % 5 == 0 { SubsystemTag::CameraV4l2 } else { SubsystemTag::AudioDsp },
            if i % 5 == 0 { Severity::Alert } else { Severity::Info },
            who,
            from,
            to,
            what,
            how,
            minutiae,
        ).unwrap();
        records.push(rec);
    }

    assert_eq!(records.len(), 15);
    assert!(TelemetryRecord::verify_chain(&records).is_ok());

    let audio_events = records.iter().filter(|r| r.subsystem == SubsystemTag::AudioDsp).count();
    let camera_events = records.iter().filter(|r| r.subsystem == SubsystemTag::CameraV4l2).count();
    assert_eq!(audio_events, 12);
    assert_eq!(camera_events, 3);
}
