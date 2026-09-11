//! Sentry Unified Hardware Sentinel
//! Unified orchestration for audio and optical sensing hardware across all platforms.

use crate::audio::SentryAudioSentinel;
use crate::camera::SentryCameraSentinel;
use crate::factory::{HardwareFactory, HardwareProfile};
use sentry_core::SentryResult;

/// Unified physical / synthetic hardware sentinel.
pub struct SentryHardwareSentinel {
    pub audio: SentryAudioSentinel,
    pub camera: SentryCameraSentinel,
}

impl SentryHardwareSentinel {
    pub fn new(audio_device: &str, camera_path: &str) -> Self {
        Self {
            audio: SentryAudioSentinel::with_device(audio_device),
            camera: SentryCameraSentinel::new(camera_path),
        }
    }

    /// Retrieve the complete hardware profile of all initialized drivers.
    pub fn profile(&self) -> HardwareProfile {
        HardwareFactory::detect_profile(self.audio.device_name(), self.camera.device_path())
    }

    /// Play an acoustic warning tone.
    pub fn alarm(&self) -> SentryResult<()> {
        self.audio.play_warning_chime()
    }
}
