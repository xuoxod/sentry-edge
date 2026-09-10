//! Acoustic RMS Audio Analyzer & Spike Detector

/// Sliding window acoustic energy and decibel SPL analyzer.
#[derive(Debug, Clone)]
pub struct AcousticAnalyzer {
    pub baseline_db: f32,
    pub trigger_delta_db: f32,
    pub sample_rate: u32,
    pub history: Vec<f32>,
    pub max_history: usize,
}

impl AcousticAnalyzer {
    pub fn new(baseline_db: f32, trigger_delta_db: f32, sample_rate: u32) -> Self {
        Self {
            baseline_db,
            trigger_delta_db,
            sample_rate,
            history: Vec::with_capacity(50),
            max_history: 50,
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

    /// Ingest a new audio buffer and determine if an acoustic spike threshold is breached.
    pub fn ingest_samples(&mut self, pcm_samples: &[i16]) -> Option<f32> {
        let current_db = self.compute_db_spl(pcm_samples);
        self.history.push(current_db);
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }

        // Dynamically compute baseline if history is sufficient
        if self.history.len() >= 10 {
            let avg: f32 = self.history.iter().sum::<f32>() / self.history.len() as f32;
            self.baseline_db = avg;
        }

        let delta = current_db - self.baseline_db;
        if delta >= self.trigger_delta_db && current_db > 40.0 {
            Some(current_db)
        } else {
            None
        }
    }
}
