//! # Sentry Hardware Abstraction Layer (`sentry-hardware`)
//! Platform-agnostic HAL supporting Linux (ALSA / V4L2), macOS (CoreAudio / AVFoundation),
//! Windows (WASAPI / MediaFoundation), and universal procedural synthetic simulation.

pub mod audio;
pub mod camera;
pub mod factory;
pub mod linux;
pub mod macos;
pub mod procedural;
pub mod sentinel;
pub mod traits;
pub mod windows;

pub use audio::SentryAudioSentinel;
pub use camera::SentryCameraSentinel;
pub use factory::{HardwareFactory, HardwareProfile};
pub use procedural::{ProceduralAudioInput, ProceduralAudioOutput, ProceduralCamera};
pub use sentinel::SentryHardwareSentinel;
pub use traits::{
    AudioInputDevice, AudioOutputDevice, CameraDevice, DriverInfo, DriverType,
};
