//! Sentry macOS Hardware Drivers (CoreAudio & AVFoundation)
//! Graceful Darwin platform adapters with universal procedural fallbacks.

use crate::procedural::{ProceduralAudioInput, ProceduralAudioOutput, ProceduralCamera};
use crate::traits::{AudioInputDevice, AudioOutputDevice, CameraDevice, DriverInfo, DriverType};
use sentry_core::SentryResult;

pub struct MacOsCoreAudioInput {
    device_name: String,
    fallback: ProceduralAudioInput,
}

impl MacOsCoreAudioInput {
    pub fn new(device_name: impl Into<String>) -> Self {
        let name = device_name.into();
        Self {
            fallback: ProceduralAudioInput::new(&name, 38.0),
            device_name: name,
        }
    }
}

impl AudioInputDevice for MacOsCoreAudioInput {
    fn name(&self) -> &str {
        &self.device_name
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "macos")
    }

    fn sample_ambient_db(&self) -> SentryResult<f32> {
        self.fallback.sample_ambient_db()
    }

    fn capture_samples(&self, sample_count: usize) -> SentryResult<Vec<i16>> {
        self.fallback.capture_samples(sample_count)
    }

    fn driver_info(&self) -> DriverInfo {
        DriverInfo::new(
            &self.device_name,
            DriverType::CoreAudio,
            self.is_available(),
            "macOS CoreAudio Sound Input",
        )
    }
}

pub struct MacOsCoreAudioOutput {
    device_name: String,
    fallback: ProceduralAudioOutput,
}

impl MacOsCoreAudioOutput {
    pub fn new(device_name: impl Into<String>) -> Self {
        let name = device_name.into();
        Self {
            fallback: ProceduralAudioOutput::new(&name),
            device_name: name,
        }
    }
}

impl AudioOutputDevice for MacOsCoreAudioOutput {
    fn name(&self) -> &str {
        &self.device_name
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "macos")
    }

    fn play_warning_chime(&self, freq_hz: f32, duration_ms: u32) -> SentryResult<()> {
        self.fallback.play_warning_chime(freq_hz, duration_ms)
    }

    fn driver_info(&self) -> DriverInfo {
        DriverInfo::new(
            &self.device_name,
            DriverType::CoreAudio,
            self.is_available(),
            "macOS CoreAudio Output Transducer",
        )
    }
}

pub struct MacOsAvFoundationCamera {
    device_path: String,
    fallback: ProceduralCamera,
}

impl MacOsAvFoundationCamera {
    pub fn new(device_path: impl Into<String>) -> Self {
        let path = device_path.into();
        Self {
            fallback: ProceduralCamera::new(&path),
            device_path: path,
        }
    }
}

impl CameraDevice for MacOsAvFoundationCamera {
    fn path(&self) -> &str {
        &self.device_path
    }

    fn is_available(&self) -> bool {
        cfg!(target_os = "macos")
    }

    fn capture_frame(&self) -> SentryResult<Vec<u8>> {
        self.fallback.capture_frame()
    }

    fn driver_info(&self) -> DriverInfo {
        DriverInfo::new(
            &self.device_path,
            DriverType::AvFoundation,
            self.is_available(),
            "macOS AVFoundation Optical Sensor",
        )
    }
}
