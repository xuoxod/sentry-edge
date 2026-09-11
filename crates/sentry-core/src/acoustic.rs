//! Acoustic RMS Audio Analyzer & Spike Detector

/// Sliding window acoustic energy and decibel SPL analyzer with adaptive DSP baseline.
#[derive(Debug, Clone)]
pub struct AcousticAnalyzer {
    pub baseline_db: f32,
    pub trigger_delta_db: f32,
    pub sample_rate: u32,
    pub history: Vec<f32>,
    pub max_history: usize,
    pub is_initialized: bool,
}

impl AcousticAnalyzer {
    pub fn new(baseline_db: f32, trigger_delta_db: f32, sample_rate: u32) -> Self {
        Self {
            baseline_db,
            trigger_delta_db,
            sample_rate,
            history: Vec::with_capacity(50),
            max_history: 50,
            is_initialized: false,
        }
    }

    /// Compute Root-Mean-Square (RMS) and decibel SPL from raw 16-bit PCM audio samples.
    pub fn compute_db_spl(&self, pcm_samples: &[i16]) -> f32 {
        if pcm_samples.is_empty() {
            return 0.0;
        }

        let sum_sq: f64 = pcm_samples
            .iter()
            .map(|&s| {
                let norm = s as f64 / 32768.0;
                norm * norm
            })
            .sum();

        let rms = (sum_sq / pcm_samples.len() as f64).sqrt();
        if rms <= 1e-6 {
            0.0
        } else {
            // Standard sound pressure level scaling (0 dB to ~100 dB SPL reference)
            let raw_db = 20.0 * rms.log10();
            ((raw_db + 90.0) as f32).clamp(0.0, 120.0)
        }
    }

    /// Compute peak transient decibel level over fast 128-sample micro-windows (~16ms).
    /// Critical for capturing transient impulses (claps, gunshots, glass shatter, door slams).
    pub fn compute_peak_transient_db(&self, pcm_samples: &[i16]) -> f32 {
        if pcm_samples.is_empty() {
            return 0.0;
        }

        let window_size = 128;
        let mut max_rms: f64 = 0.0;

        for chunk in pcm_samples.chunks(window_size) {
            let sum_sq: f64 = chunk
                .iter()
                .map(|&s| {
                    let norm = s as f64 / 32768.0;
                    norm * norm
                })
                .sum();
            let rms = (sum_sq / chunk.len() as f64).sqrt();
            if rms > max_rms {
                max_rms = rms;
            }
        }

        if max_rms <= 1e-6 {
            0.0
        } else {
            let raw_db = 20.0 * max_rms.log10();
            ((raw_db + 90.0) as f32).clamp(0.0, 120.0)
        }
    }

    /// Ingest a new audio buffer and determine if an acoustic spike threshold is breached.
    pub fn ingest_samples(&mut self, pcm_samples: &[i16]) -> Option<f32> {
        let ambient_db = self.compute_db_spl(pcm_samples);
        let peak_db = self.compute_peak_transient_db(pcm_samples);

        if !self.is_initialized {
            self.baseline_db = ambient_db;
            self.is_initialized = true;
        }

        let delta = peak_db - self.baseline_db;

        // Check if transient peak breaches trigger threshold (ignoring background whisper < 45 dB)
        let is_spike = delta >= self.trigger_delta_db && peak_db > 45.0;

        // Adaptively update baseline via Exponential Moving Average (EMA) using ambient sustained level
        if !is_spike {
            let alpha = 0.15; // Smooth adaptation factor
            self.baseline_db = (1.0 - alpha) * self.baseline_db + alpha * ambient_db;
        }

        self.history.push(ambient_db);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        if is_spike {
            Some(peak_db)
        } else {
            None
        }
    }
}

