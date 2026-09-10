//! TDD Test: Security Tamper Detection, Signature Forgery & Adversarial Attacks

use sentry_core::{IncidentSeverity, SentryAlert, SentryIncidentType};
use uuid::Uuid;

#[test]
fn test_rw_alert_signature_verification_success() {
    let alert = SentryAlert::new(
        Uuid::new_v4(),
        "Server-Room-Sentinel",
        IncidentSeverity::High,
        SentryIncidentType::AcousticSpike {
            peak_db: 85.0,
            baseline_db: 35.0,
            delta_db: 50.0,
        },
        "Acoustic threshold breach",
        None,
    );

    assert!(alert.verify_signature(), "Unmodified alert signature must verify successfully");
}

#[test]
fn test_attack_alert_payload_tamper_detection() {
    let mut alert = SentryAlert::new(
        Uuid::new_v4(),
        "Server-Room-Sentinel",
        IncidentSeverity::High,
        SentryIncidentType::AcousticSpike {
            peak_db: 85.0,
            baseline_db: 35.0,
            delta_db: 50.0,
        },
        "Acoustic threshold breach",
        None,
    );

    // Adversary tampers with the description to hide the severity
    alert.description = "Nothing to see here (TAMPERED)".to_string();

    assert!(!alert.verify_signature(), "Tampered alert payload MUST fail signature verification");
}

#[test]
fn test_attack_node_label_spoofing_detection() {
    let mut alert = SentryAlert::new(
        Uuid::new_v4(),
        "Legitimate-Node-01",
        IncidentSeverity::Critical,
        SentryIncidentType::TamperDetected {
            reason: "Enclosure opened".to_string(),
        },
        "Physical tamper",
        None,
    );

    // Adversary spoofs the node label to frame a different machine
    alert.node_label = "Framed-Node-02".to_string();

    assert!(!alert.verify_signature(), "Spoofed node label MUST fail signature verification");
}

#[test]
fn test_attack_corrupted_signature_detection() {
    let mut alert = SentryAlert::new(
        Uuid::new_v4(),
        "Sentinel-Alpha",
        IncidentSeverity::Warning,
        SentryIncidentType::PeriodicHeartbeat,
        "Heartbeat ok",
        None,
    );

    // Corrupt one character in the hex signature
    alert.event_signature_sha256 = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

    assert!(!alert.verify_signature(), "Forged signature MUST fail verification");
}
