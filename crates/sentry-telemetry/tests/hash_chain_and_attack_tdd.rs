use sentry_telemetry::{
    FromProvenance, HowProvenance, LogSanitizer, MinutiaeMetadata, Severity,
    SubsystemTag, TelemetryEngine, TelemetryRecord, ToProvenance, WhatTelemetry, WhoProvenance,
};

fn dummy_provenance(summary: &str) -> (WhoProvenance, FromProvenance, ToProvenance, WhatTelemetry, HowProvenance, MinutiaeMetadata) {
    (
        WhoProvenance {
            identity: "daemon".into(),
            token_prefix: "tok".into(),
            session_id: "sess".into(),
            peer_id: None,
        },
        FromProvenance {
            source_device: "dev".into(),
            thread_id: "1".into(),
            physical_addr: None,
            endpoint: "ep".into(),
        },
        ToProvenance {
            destination_hardware: None,
            remote_relay: "relay".into(),
            database_wal: "db".into(),
            client_ui: None,
        },
        WhatTelemetry {
            summary: summary.into(),
            rms_db: Some(45.0),
            baseline_db: Some(40.0),
            delta_db: Some(5.0),
            frames_captured: 0,
            shutter_latency_ns: 0,
            shutter_latency_ps: 0,
            payload_bytes: 64,
            payload_sha256: "hash123".into(),
            duration_ns: 1000,
            duration_ps: 1_000_000,
        },
        HowProvenance {
            protocol: "proto".into(),
            transport: "trans".into(),
            cipher: "ciph".into(),
            compression: None,
        },
        MinutiaeMetadata {
            cpu_rss_mb: 10.0,
            dsp_ema_alpha: 0.15,
            wal_page_count: 1,
            sqlite_commit_ns: 500,
            network_rtt_ms: 1.0,
        },
    )
}

#[test]
fn test_rw_cryptographic_hash_chain_verification_success() {
    let engine = TelemetryEngine::new("crunchbang-laptop", false);
    let mut records = Vec::new();

    for i in 1..=20 {
        let (who, from, to, what, how, minutiae) = dummy_provenance(&format!("Sequential event #{}", i));
        let record = engine.log(
            SubsystemTag::AudioDsp,
            Severity::Info,
            who,
            from,
            to,
            what,
            how,
            minutiae,
        ).unwrap();
        records.push(record);
    }

    assert_eq!(records.len(), 20);
    assert!(TelemetryRecord::verify_chain(&records).is_ok());
}

#[test]
fn test_attack_log_injection_crlf_sanitization() {
    let malicious_summary = "Normal Event\r\n[ALERT] FAKE ROOT ACCESS COMPROMISED\r\n[DEBUG] Ignored line";
    let sanitized = LogSanitizer::sanitize(malicious_summary);

    assert!(!sanitized.contains('\r'), "Carriage return must be neutralized");
    assert!(!sanitized.contains('\n'), "Newline must be neutralized");
    assert!(LogSanitizer::validate_no_crlf(&sanitized));

    let (who, from, to, what, how, minutiae) = dummy_provenance(malicious_summary);
    let record = TelemetryRecord::new(
        1,
        "node",
        SubsystemTag::AuthGate,
        Severity::Warn,
        who,
        from,
        to,
        what,
        how,
        minutiae,
        "GENESIS",
    );

    assert!(!record.what.summary.contains('\n'));
    assert!(record.verify_integrity());
}

#[test]
fn test_attack_hash_chain_tamper_detected() {
    let engine = TelemetryEngine::new("crunchbang-laptop", false);
    let mut records = Vec::new();

    for i in 1..=10 {
        let (who, from, to, what, how, minutiae) = dummy_provenance(&format!("Event #{}", i));
        let record = engine.log(
            SubsystemTag::AudioDsp,
            Severity::Info,
            who,
            from,
            to,
            what,
            how,
            minutiae,
        ).unwrap();
        records.push(record);
    }

    // Tamper with record #5 payload
    records[4].what.summary = "TAMPERED EXPLOIT INSERTION".to_string();

    let result = TelemetryRecord::verify_chain(&records);
    assert!(result.is_err(), "Hash chain verification must detect modified summary");
}

#[test]
fn test_attack_hash_chain_reorder_detected() {
    let engine = TelemetryEngine::new("crunchbang-laptop", false);
    let mut records = Vec::new();

    for i in 1..=10 {
        let (who, from, to, what, how, minutiae) = dummy_provenance(&format!("Event #{}", i));
        let record = engine.log(
            SubsystemTag::AudioDsp,
            Severity::Info,
            who,
            from,
            to,
            what,
            how,
            minutiae,
        ).unwrap();
        records.push(record);
    }

    // Swap record #2 and record #3
    records.swap(1, 2);

    let result = TelemetryRecord::verify_chain(&records);
    assert!(result.is_err(), "Hash chain verification must detect reordered records");
}
