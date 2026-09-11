//! TDD Test: Hardware Abstraction Layer & Cross-Platform Profile CLI Integration

use sentry_core::platform::{PlatformInfo, PlatformPaths};
use sentry_hardware::{HardwareFactory, SentryHardwareSentinel};

#[test]
fn test_rw_hal_hardware_profile_inspection() {
    let platform = PlatformInfo::detect();
    assert!(!platform.os.is_empty());
    assert!(!platform.arch.is_empty());

    let profile = HardwareFactory::detect_profile("default", "/dev/video0");
    assert_eq!(profile.os, platform.os);
    assert_eq!(profile.arch, platform.arch);
    assert!(!profile.audio_input.details.is_empty());
    assert!(!profile.camera.details.is_empty());
}

#[test]
fn test_rw_unified_hardware_sentinel_lifecycle() {
    let sentinel = SentryHardwareSentinel::new("default", "/dev/video0");
    let profile = sentinel.profile();
    assert!(!profile.audio_input.name.is_empty());

    // Test ambient decibel sampling
    let db = sentinel.audio.sample_ambient_db().unwrap();
    assert!(db >= 0.0);

    // Test live sample capture
    let samples = sentinel.audio.capture_live_samples(512);
    assert!(samples.len() >= 512);

    // Test camera burst
    let burst = sentinel.camera.capture_burst(3).unwrap();
    assert_eq!(burst.len(), 3);
}

#[test]
fn test_rw_platform_directories_exist_and_accessible() {
    let config_path = PlatformPaths::default_config_path();
    let ledger_path = PlatformPaths::default_ledger_path();
    let audit_path = PlatformPaths::default_audit_log_path();

    assert!(config_path.to_string_lossy().contains("sentry.toml"));
    assert!(ledger_path.to_string_lossy().contains("sentry_ledger.db"));
    assert!(audit_path.to_string_lossy().contains("sentry_audit.jsonl"));
}
