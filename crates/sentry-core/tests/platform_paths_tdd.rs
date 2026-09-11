//! TDD Suite for Platform Detection and Cross-Platform Path Resolution

use sentry_core::platform::{PlatformInfo, PlatformPaths};
use std::path::Path;

#[test]
fn test_rw_platform_detection() {
    let info = PlatformInfo::detect();
    assert!(!info.os.is_empty(), "OS should not be empty");
    assert!(!info.arch.is_empty(), "Arch should not be empty");
    assert!(!info.family.is_empty(), "Family should not be empty");

    let summary = info.display_summary();
    assert!(summary.contains(&info.os));
    assert!(summary.contains(&info.arch));
}

#[test]
fn test_rw_platform_paths_resolution() {
    let config_dir = PlatformPaths::config_dir();
    assert!(!config_dir.to_string_lossy().is_empty());

    let default_config = PlatformPaths::default_config_path();
    assert!(default_config.ends_with("sentry.toml"));

    let data_dir = PlatformPaths::data_dir();
    assert!(!data_dir.to_string_lossy().is_empty());

    let log_dir = PlatformPaths::log_dir();
    assert!(!log_dir.to_string_lossy().is_empty());

    let ledger = PlatformPaths::default_ledger_path();
    assert!(ledger.ends_with("sentry_ledger.db"));

    let audit = PlatformPaths::default_audit_log_path();
    assert!(audit.ends_with("sentry_audit.jsonl"));
}

#[test]
fn test_ec_platform_paths_safe_validation() {
    // Valid relative and absolute paths
    assert!(PlatformPaths::validate_safe_path(Path::new("./data/ledger.db")).is_ok());
    assert!(PlatformPaths::validate_safe_path(Path::new("/var/log/sentry/audit.jsonl")).is_ok());

    // Path traversal attack detection
    assert!(PlatformPaths::validate_safe_path(Path::new("../../../etc/shadow")).is_err());
    assert!(PlatformPaths::validate_safe_path(Path::new("./data/../../secret")).is_err());

    // Forbidden Windows device names detection
    assert!(PlatformPaths::validate_safe_path(Path::new("CON.txt")).is_err());
    assert!(PlatformPaths::validate_safe_path(Path::new("NUL")).is_err());
    assert!(PlatformPaths::validate_safe_path(Path::new("AUX.log")).is_err());
    assert!(PlatformPaths::validate_safe_path(Path::new("com1.dat")).is_err());
}
