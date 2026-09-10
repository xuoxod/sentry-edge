//! TDD Test: Sentry Hardware Sentinel Engine

use sentry_hardware::{SentryAudioSentinel, SentryCameraSentinel};

#[test]
fn test_sentry_audio_sentinel_chime_and_meter() {
    let audio = SentryAudioSentinel::new();
    assert!(audio.play_warning_chime().is_ok());
    let db = audio.sample_ambient_db();
    assert!(db.is_ok());
    assert!(db.unwrap() >= 0.0);
}

#[test]
fn test_sentry_camera_sentinel_burst() {
    let camera = SentryCameraSentinel::new("/dev/video0");
    let burst = camera.capture_burst(2);
    assert!(burst.is_ok());
    let frames = burst.unwrap();
    assert_eq!(frames.len(), 2);
    assert!(!frames[0].is_empty());
}
