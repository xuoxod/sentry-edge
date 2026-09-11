//! Sentry Hardware Abstraction Layer (HAL) Traits & Driver Metadata
//! Enterprise-grade platform-agnostic interface for acoustic decibel watchdogs and camera sentinels.

use sentry_core::SentryResult;
use serde::{Deserialize, Serialize};

/// Categorization of active hardware and synthetic HAL drivers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriverType {
    Alsa,
    PulseAudio,
    PipeWire,
    CoreAudio,
    Wasapi,
    V4l2,
    AvFoundation,
    MediaFoundation,
    ProceduralSynthetic,
    Null,
}

/// Metadata describing an initialized HAL driver.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriverInfo {
    pub name: String,
    pub driver_type: DriverType,
    pub platform: String,
    pub is_hardware: bool,
    pub details: String,
}

impl DriverInfo {
    pub fn new(
        name: impl Into<String>,
        driver_type: DriverType,
        is_hardware: bool,
        details: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            driver_type,
            platform: std::env::consts::OS.to_string(),
            is_hardware,
            details: details.into(),
        }
    }
}

/// Trait for ambient acoustic input devices (Microphones, DSP sensors, Sound Cards).
pub trait AudioInputDevice: Send + Sync {
    /// Friendly name or identifier of the audio input device.
    fn name(&self) -> &str;

    /// Returns true if physical hardware is detected and accessible.
    fn is_available(&self) -> bool;

    /// Sample current ambient noise sound pressure level in dB SPL.
    fn sample_ambient_db(&self) -> SentryResult<f32>;

    /// Capture raw 16-bit signed PCM audio samples (single channel, 8000Hz-48000Hz).
    fn capture_samples(&self, sample_count: usize) -> SentryResult<Vec<i16>>;

    /// Return driver metadata.
    fn driver_info(&self) -> DriverInfo;
}

/// Trait for audio output / attention alarm transducers (Sirens, Buzzers, Speakers, Chimes).
pub trait AudioOutputDevice: Send + Sync {
    /// Friendly name or identifier of the audio output device.
    fn name(&self) -> &str;

    /// Returns true if physical output hardware is accessible.
    fn is_available(&self) -> bool;

    /// Play an acoustic warning tone / attention chime at specified frequency and duration.
    fn play_warning_chime(&self, freq_hz: f32, duration_ms: u32) -> SentryResult<()>;

    /// Return driver metadata.
    fn driver_info(&self) -> DriverInfo;
}

/// Trait for camera capture sensors (V4L2, AVFoundation, MediaFoundation, Synthetic Test Pattern).
pub trait CameraDevice: Send + Sync {
    /// Path or descriptor to the camera sensor.
    fn path(&self) -> &str;

    /// Returns true if physical optical sensor is accessible.
    fn is_available(&self) -> bool;

    /// Capture a single high-resolution JPEG frame payload.
    fn capture_frame(&self) -> SentryResult<Vec<u8>>;

    /// Capture a rapid multi-frame optical burst upon security trigger.
    fn capture_burst(&self, count: usize) -> SentryResult<Vec<Vec<u8>>> {
        let mut frames = Vec::with_capacity(count);
        for _ in 0..count {
            frames.push(self.capture_frame()?);
        }
        Ok(frames)
    }

    /// Return driver metadata.
    fn driver_info(&self) -> DriverInfo;
}
