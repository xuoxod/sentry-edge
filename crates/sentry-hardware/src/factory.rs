//! Sentry HAL Factory & Platform Auto-Detector
//! Seamless runtime dynamic hardware driver instantiation with zero panics.

use crate::traits::{AudioInputDevice, AudioOutputDevice, CameraDevice, DriverInfo};
use serde::{Deserialize, Serialize};

/// Comprehensive report of the active hardware and platform detection profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub os: String,
    pub arch: String,
    pub audio_input: DriverInfo,
    pub audio_output: DriverInfo,
    pub camera: DriverInfo,
}

/// Unified factory for resolving HAL drivers across any operating system.
pub struct HardwareFactory;

impl HardwareFactory {
    /// Instantiate the best audio input driver for the host platform.
    pub fn create_audio_input(device_name: &str) -> Box<dyn AudioInputDevice> {
        #[cfg(target_os = "linux")]
        {
            Box::new(crate::linux::LinuxAlsaAudioInput::new(device_name))
        }

        #[cfg(target_os = "macos")]
        {
            Box::new(crate::macos::MacOsCoreAudioInput::new(device_name))
        }

        #[cfg(target_os = "windows")]
        {
            Box::new(crate::windows::WindowsWasapiAudioInput::new(device_name))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Box::new(crate::procedural::ProceduralAudioInput::new(device_name, 38.5))
        }
    }

    /// Instantiate the best audio output driver for the host platform.
    pub fn create_audio_output(device_name: &str) -> Box<dyn AudioOutputDevice> {
        #[cfg(target_os = "linux")]
        {
            Box::new(crate::linux::LinuxAlsaAudioOutput::new(device_name))
        }

        #[cfg(target_os = "macos")]
        {
            Box::new(crate::macos::MacOsCoreAudioOutput::new(device_name))
        }

        #[cfg(target_os = "windows")]
        {
            Box::new(crate::windows::WindowsWasapiAudioOutput::new(device_name))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Box::new(crate::procedural::ProceduralAudioOutput::new(device_name))
        }
    }

    /// Instantiate the best camera capture driver for the host platform.
    pub fn create_camera(device_path: &str) -> Box<dyn CameraDevice> {
        #[cfg(target_os = "linux")]
        {
            Box::new(crate::linux::LinuxV4l2Camera::new(device_path))
        }

        #[cfg(target_os = "macos")]
        {
            Box::new(crate::macos::MacOsAvFoundationCamera::new(device_path))
        }

        #[cfg(target_os = "windows")]
        {
            Box::new(crate::windows::WindowsMediaFoundationCamera::new(device_path))
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Box::new(crate::procedural::ProceduralCamera::new(device_path))
        }
    }

    /// Detect and build the full hardware profile.
    pub fn detect_profile(audio_device: &str, camera_path: &str) -> HardwareProfile {
        let audio_in = Self::create_audio_input(audio_device);
        let audio_out = Self::create_audio_output(audio_device);
        let camera = Self::create_camera(camera_path);

        HardwareProfile {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            audio_input: audio_in.driver_info(),
            audio_output: audio_out.driver_info(),
            camera: camera.driver_info(),
        }
    }
}
