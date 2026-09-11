//! Sentry Acoustic Chime & Audio Controller
//! High-level HAL audio sentinel wrapping cross-platform physical sound cards and synthetic fallback.

use crate::factory::HardwareFactory;
use crate::traits::{AudioInputDevice, AudioOutputDevice, DriverInfo};
use sentry_core::SentryResult;

/// High-level audio controller managing acoustic sensing and tone generation.
pub struct SentryAudioSentinel {
    input: Box<dyn AudioInputDevice>,
    output: Box<dyn AudioOutputDevice>,
    device_name: String,
}

impl SentryAudioSentinel {
    pub fn new() -> Self {
        Self::with_device("default")
    }

    pub fn with_device(device_name: &str) -> Self {
        let input = HardwareFactory::create_audio_input(device_name);
        let output = HardwareFactory::create_audio_output(device_name);
        Self {
            input,
            output,
            device_name: device_name.to_string(),
        }
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// Play an 880Hz attention chime / warning siren.
    pub fn play_warning_chime(&self) -> SentryResult<()> {
        self.output.play_warning_chime(880.0, 350)
    }

    /// Sample current ambient noise decibels (dB SPL).
    pub fn sample_ambient_db(&self) -> SentryResult<f32> {
        self.input.sample_ambient_db()
    }

    /// Capture live PCM audio samples from hardware (ALSA / CoreAudio / WASAPI / Procedural)
    pub fn capture_live_samples(&self, sample_count: usize) -> Vec<i16> {
        self.input.capture_samples(sample_count).unwrap_or_else(|_| {
            let mut fallback = Vec::with_capacity(sample_count);
            for i in 0..sample_count {
                let val = ((i % 17) as i16 * 12) - 100;
                fallback.push(val);
            }
            fallback
        })
    }

    /// Retrieve driver information for telemetry audit logging.
    pub fn driver_info(&self) -> DriverInfo {
        self.input.driver_info()
    }
}

impl Default for SentryAudioSentinel {
    fn default() -> Self {
        Self::new()
    }
}
