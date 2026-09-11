//! Sentry Camera Capture & Optical Burst Controller
//! High-level HAL camera sentinel wrapping cross-platform physical cameras and synthetic test patterns.

use crate::factory::HardwareFactory;
use crate::traits::{CameraDevice, DriverInfo};
use sentry_core::SentryResult;

/// High-level camera sentinel managing frame capture and rapid security bursts.
pub struct SentryCameraSentinel {
    camera: Box<dyn CameraDevice>,
    device_path: String,
}

impl SentryCameraSentinel {
    pub fn new(device_path: impl Into<String>) -> Self {
        let path = device_path.into();
        let camera = HardwareFactory::create_camera(&path);
        Self {
            camera,
            device_path: path,
        }
    }

    pub fn device_path(&self) -> &str {
        &self.device_path
    }

    /// Capture a single high-resolution JPEG frame.
    pub fn capture_frame(&self) -> SentryResult<Vec<u8>> {
        self.camera.capture_frame()
    }

    /// Capture a multi-frame burst upon security trigger.
    pub fn capture_burst(&self, count: usize) -> SentryResult<Vec<Vec<u8>>> {
        self.camera.capture_burst(count)
    }

    /// Retrieve driver information for telemetry audit logging.
    pub fn driver_info(&self) -> DriverInfo {
        self.camera.driver_info()
    }
}
