//! TDD Test: Configuration RW, EC, and Adversarial Attack Suite

use sentry_core::config::SentryConfig;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_rw_config_template_generation_and_roundtrip() {
    let template = SentryConfig::default_template();
    assert!(template.contains("[node]"));
    assert!(template.contains("[network]"));
    assert!(template.contains("[hardware]"));
    assert!(template.contains("[telepresence]"));
    assert!(template.contains("[storage]"));

    let config = SentryConfig::from_toml_str(&template).expect("Default template must parse cleanly");
    assert_eq!(config.node.label, "Server-Room-Sentinel (hyperion-prime)");
    assert_eq!(config.hardware.acoustic_trigger_delta_db, 20.0);
    assert_eq!(config.hardware.snapshot_burst_count, 5);
}

#[test]
fn test_rw_config_disk_persistence() {
    let tmp = tempdir().unwrap();
    let config_path = tmp.path().join("sentry.toml");

    let original_config = SentryConfig::default();
    let toml_str = toml::to_string(&original_config).unwrap();
    fs::write(&config_path, toml_str).unwrap();

    let loaded = SentryConfig::load_from_file(&config_path).expect("Must load from disk");
    assert_eq!(original_config, loaded);
}

#[test]
fn test_ec_config_boundary_values() {
    // 1. Min valid trigger (0.1 dB)
    let mut config = SentryConfig::default();
    config.hardware.acoustic_trigger_delta_db = 0.1;
    config.hardware.snapshot_burst_count = 1;
    assert!(config.validate().is_ok());

    // 2. Max valid burst (50 frames)
    config.hardware.snapshot_burst_count = 50;
    assert!(config.validate().is_ok());
}

#[test]
fn test_attack_config_malformed_syntax() {
    let bad_toml = "this is not valid toml = [[[";
    let res = SentryConfig::from_toml_str(bad_toml);
    assert!(res.is_err(), "Malformed TOML syntax must be rejected");
}

#[test]
fn test_attack_config_path_traversal() {
    let mut config = SentryConfig::default();
    config.storage.ledger_db_path = "../../etc/shadow".to_string();

    let res = config.validate();
    assert!(res.is_err(), "Path traversal in database path must be blocked");
    assert!(res.unwrap_err().to_string().contains("path traversal"));
}

#[test]
fn test_attack_config_invalid_relay_protocol() {
    let mut config = SentryConfig::default();
    config.network.relay_url = "http://malicious-http-trap.com".to_string();

    let res = config.validate();
    assert!(res.is_err(), "Non-WebSocket protocols (http://) must be rejected");
}

#[test]
fn test_attack_config_extreme_trigger_overflow() {
    let mut config = SentryConfig::default();
    config.hardware.acoustic_trigger_delta_db = 99999.0;

    let res = config.validate();
    assert!(res.is_err(), "Out-of-bound acoustic trigger must be rejected");
}
