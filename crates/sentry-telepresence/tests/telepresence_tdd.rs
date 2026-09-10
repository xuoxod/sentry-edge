//! TDD Test: LiveKit WebRTC Session & Intercom Bridge

use sentry_telepresence::{SentryIntercomBridge, SentryLiveKitSession};

#[test]
fn test_livekit_session_creation_and_token() {
    let session = SentryLiveKitSession::new(
        "https://sfu.example.com:7880",
        "sentry-room-alpha",
        "operator-mobile",
    );

    assert_eq!(session.room_name, "sentry-room-alpha");
    assert!(!session.token.is_empty(), "Token must be generated");
    assert!(session.health_check().unwrap());
}

#[test]
fn test_intercom_bridge_lifecycle() {
    let mut intercom = SentryIntercomBridge::new();
    assert!(!intercom.is_active);

    intercom.open_operator_intercom("rick-mobile").unwrap();
    assert!(intercom.is_active);
    assert_eq!(intercom.active_operator, Some("rick-mobile".to_string()));

    intercom.close_intercom().unwrap();
    assert!(!intercom.is_active);
    assert_eq!(intercom.active_operator, None);
}
