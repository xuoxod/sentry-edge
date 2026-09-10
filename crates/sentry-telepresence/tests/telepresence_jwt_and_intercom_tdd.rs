//! TDD Test: LiveKit SFU JWT Validation, HMAC-SHA256 & Intercom Audio Bridge

use sentry_telepresence::{SentryIntercomBridge, SentryLiveKitSession};

#[test]
fn test_rw_livekit_custom_credentials_and_jwt() {
    let session = SentryLiveKitSession::with_credentials(
        "https://sfu.example.com:7880",
        "production-datacenter-rack-01",
        "operator-rick",
        Some("my-custom-api-key"),
        Some("my-custom-api-secret"),
    );

    assert_eq!(session.room_name, "production-datacenter-rack-01");
    assert!(session.token.starts_with("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    assert!(session.health_check().is_ok());
}

#[test]
fn test_ec_livekit_default_dev_fallback() {
    let session = SentryLiveKitSession::new(
        "https://sfu.example.com:7880",
        "room-test",
        "operator-dev",
    );

    assert!(!session.token.is_empty());
    assert!(session.health_check().is_ok());
}

#[test]
fn test_rw_intercom_multiple_operator_handoff() {
    let mut intercom = SentryIntercomBridge::new();
    assert!(!intercom.is_active);

    // Operator A takes the mic
    intercom.open_operator_intercom("operator-rick").unwrap();
    assert!(intercom.is_active);
    assert_eq!(intercom.active_operator, Some("operator-rick".to_string()));

    // Operator B takes over
    intercom.open_operator_intercom("operator-guard").unwrap();
    assert_eq!(intercom.active_operator, Some("operator-guard".to_string()));

    // Session closes
    intercom.close_intercom().unwrap();
    assert!(!intercom.is_active);
    assert_eq!(intercom.active_operator, None);
}
