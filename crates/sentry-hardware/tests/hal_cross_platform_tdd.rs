//! TDD Suite for Platform-Agnostic HAL Architecture, Multi-OS Drivers & Procedural Fallbacks

use sentry_hardware::{
    AudioInputDevice, AudioOutputDevice, CameraDevice, DriverType,
    ProceduralAudioInput, ProceduralAudioOutput, ProceduralCamera, SentryHardwareSentinel,
};

#[test]
fn test_rw_procedural_audio_input_and_output() {
    let audio_in = ProceduralAudioInput::new("test-mic", 42.0);
    assert_eq!(audio_in.name(), "test-mic");
    assert!(audio_in.is_available());

    let db = audio_in.sample_ambient_db().unwrap();
    assert!(db >= 40.0 && db <= 44.0, "Expected db near 42.0, got {}", db);

    let samples = audio_in.capture_samples(256).unwrap();
    assert_eq!(samples.len(), 256);

    let info = audio_in.driver_info();
    assert_eq!(info.driver_type, DriverType::ProceduralSynthetic);
    assert!(!info.is_hardware);

    let audio_out = ProceduralAudioOutput::new("test-speaker");
    assert_eq!(audio_out.name(), "test-speaker");
    assert_eq!(audio_out.get_chime_count(), 0);
    assert!(audio_out.play_warning_chime(880.0, 200).is_ok());
    assert_eq!(audio_out.get_chime_count(), 1);
}

#[test]
fn test_rw_procedural_camera_jpeg_compliance() {
    let camera = ProceduralCamera::new("cam-01");
    assert_eq!(camera.path(), "cam-01");
    assert!(camera.is_available());

    let frame = camera.capture_frame().unwrap();
    // Verify standard JPEG SOI (0xFF, 0xD8) and EOI (0xFF, 0xD9)
    assert!(frame.len() > 10);
    assert_eq!(frame[0], 0xFF);
    assert_eq!(frame[1], 0xD8);
    assert_eq!(frame[frame.len() - 2], 0xFF);
    assert_eq!(frame[frame.len() - 1], 0xD9);

    let burst = camera.capture_burst(4).unwrap();
    assert_eq!(burst.len(), 4);
    for f in &burst {
        assert_eq!(f[0], 0xFF);
        assert_eq!(f[1], 0xD8);
    }
}

#[test]
fn test_rw_hardware_factory_and_profile() {
    let sentinel = SentryHardwareSentinel::new("default", "/dev/video0");
    let profile = sentinel.profile();

    assert!(!profile.os.is_empty());
    assert!(!profile.arch.is_empty());
    assert!(!profile.audio_input.name.is_empty());
    assert!(!profile.camera.name.is_empty());
    assert!(sentinel.alarm().is_ok());
}

#[test]
fn test_ec_zero_and_large_bursts() {
    let camera = ProceduralCamera::default();
    let zero_burst = camera.capture_burst(0).unwrap();
    assert!(zero_burst.is_empty());

    let large_burst = camera.capture_burst(20).unwrap();
    assert_eq!(large_burst.len(), 20);
}

#[test]
fn test_ec_audio_samples_boundary() {
    let audio = ProceduralAudioInput::default();
    let zero_samples = audio.capture_samples(0).unwrap();
    assert!(zero_samples.is_empty());

    let large_samples = audio.capture_samples(4096).unwrap();
    assert_eq!(large_samples.len(), 4096);
}
