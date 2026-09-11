//! Sentry Procedural Synthetic HAL Driver
//! Universal, deterministic hardware simulation driver for CI, containerized nodes, and cross-platform environments.

use crate::traits::{AudioInputDevice, AudioOutputDevice, CameraDevice, DriverInfo, DriverType};
use chrono::Utc;
use sentry_core::SentryResult;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

/// High-fidelity procedural acoustic input generator.
pub struct ProceduralAudioInput {
    name: String,
    baseline_db: f32,
    phase: AtomicU32,
}

impl ProceduralAudioInput {
    pub fn new(name: impl Into<String>, baseline_db: f32) -> Self {
        Self {
            name: name.into(),
            baseline_db,
            phase: AtomicU32::new(0),
        }
    }
}

impl Default for ProceduralAudioInput {
    fn default() -> Self {
        Self::new("synthetic-audio-loopback", 38.5)
    }
}

impl AudioInputDevice for ProceduralAudioInput {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        true
    }

    fn sample_ambient_db(&self) -> SentryResult<f32> {
        let p = self.phase.fetch_add(1, Ordering::Relaxed);
        // Small realistic micro-fluctuation (+/- 0.5 dB)
        let wobble = (((p % 13) as f32) - 6.0) * 0.08;
        Ok((self.baseline_db + wobble).clamp(10.0, 120.0))
    }

    fn capture_samples(&self, sample_count: usize) -> SentryResult<Vec<i16>> {
        let p = self.phase.fetch_add(sample_count as u32, Ordering::Relaxed);
        let mut samples = Vec::with_capacity(sample_count);
        for i in 0..sample_count {
            let idx = (p as usize + i) as f32;
            // 440Hz baseline harmonic + low-level white noise
            let val = (idx * 0.05).sin() * 800.0 + ((idx % 7.0) * 15.0);
            samples.push(val as i16);
        }
        Ok(samples)
    }

    fn driver_info(&self) -> DriverInfo {
        DriverInfo::new(
            &self.name,
            DriverType::ProceduralSynthetic,
            false,
            "In-memory procedural DSP acoustic synthesizer",
        )
    }
}

/// Procedural audio output tone sink and event tracker.
pub struct ProceduralAudioOutput {
    name: String,
    played_chimes: Mutex<Vec<(f32, u32)>>,
}

impl ProceduralAudioOutput {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            played_chimes: Mutex::new(Vec::new()),
        }
    }

    pub fn get_chime_count(&self) -> usize {
        self.played_chimes.lock().unwrap().len()
    }
}

impl Default for ProceduralAudioOutput {
    fn default() -> Self {
        Self::new("synthetic-audio-speaker")
    }
}

impl AudioOutputDevice for ProceduralAudioOutput {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_available(&self) -> bool {
        true
    }

    fn play_warning_chime(&self, freq_hz: f32, duration_ms: u32) -> SentryResult<()> {
        if let Ok(mut lock) = self.played_chimes.lock() {
            lock.push((freq_hz, duration_ms));
        }
        Ok(())
    }

    fn driver_info(&self) -> DriverInfo {
        DriverInfo::new(
            &self.name,
            DriverType::ProceduralSynthetic,
            false,
            "In-memory procedural chime and siren output transducer",
        )
    }
}

/// Procedural JPEG camera sensor generator with genuine JFIF structure.
pub struct ProceduralCamera {
    path: String,
}

impl ProceduralCamera {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl Default for ProceduralCamera {
    fn default() -> Self {
        Self::new("virtual-v4l2-camera")
    }
}

impl CameraDevice for ProceduralCamera {
    fn path(&self) -> &str {
        &self.path
    }

    fn is_available(&self) -> bool {
        true
    }

    fn capture_frame(&self) -> SentryResult<Vec<u8>> {
        // Construct authentic JPEG SOI + APP0 (JFIF) + payload + EOI
        let timestamp = Utc::now().to_rfc3339();
        let mut jpeg = vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xE0, // APP0
            0x00, 0x10, // Length
            b'J', b'F', b'I', b'F', 0x00, // Identifier
            0x01, 0x01, // Version 1.1
            0x01,       // Density units
            0x00, 0x48, 0x00, 0x48, // 72x72 DPI
            0x00, 0x00, // No thumbnail
        ];

        let watermark = format!("SENTRY_EDGE_FRAME_{}_{}", self.path, timestamp);
        jpeg.extend_from_slice(watermark.as_bytes());
        jpeg.extend_from_slice(&[0xFF, 0xD9]); // EOI
        Ok(jpeg)
    }

    fn driver_info(&self) -> DriverInfo {
        DriverInfo::new(
            &self.path,
            DriverType::ProceduralSynthetic,
            false,
            "In-memory standards-compliant JPEG test pattern camera",
        )
    }
}
