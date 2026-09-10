//! TDD Test: Acoustic Signal Simulation, Chaos Flapping & Audio Buffer Edge Cases

use sentry_core::AcousticAnalyzer;

#[test]
fn test_ec_empty_audio_buffer() {
    let analyzer = AcousticAnalyzer::new(35.0, 20.0, 48000);
    let empty_samples: Vec<i16> = vec![];
    assert_eq!(analyzer.compute_db_spl(&empty_samples), 0.0);
}

#[test]
fn test_ec_zero_amplitude_silence() {
    let analyzer = AcousticAnalyzer::new(35.0, 20.0, 48000);
    let silence = vec![0i16; 2048];
    let db = analyzer.compute_db_spl(&silence);
    assert_eq!(db, 0.0, "Zero amplitude silence should compute to 0.0 dB SPL");
}

#[test]
fn test_ec_maximum_clipping_amplitude() {
    let analyzer = AcousticAnalyzer::new(35.0, 20.0, 48000);
    let max_positive = vec![i16::MAX; 1024];
    let max_db = analyzer.compute_db_spl(&max_positive);
    assert!(max_db >= 89.0 && max_db <= 95.0, "Full-scale square wave should be near ~90 dB: {}", max_db);
}

#[test]
fn test_sim_chaos_rapid_flapping_audio_storm() {
    let mut analyzer = AcousticAnalyzer::new(35.0, 15.0, 48000);
    let mut trigger_count = 0;

    // Simulate 100 alternating cycles of silence and screaming loud bursts
    for cycle in 0..100 {
        let samples = if cycle % 2 == 0 {
            vec![100i16; 512] // Quiet
        } else {
            vec![30000i16; 512] // Extreme sound spike
        };

        if analyzer.ingest_samples(&samples).is_some() {
            trigger_count += 1;
        }
    }

    assert!(trigger_count > 0, "Spike triggers must successfully intercept rapid acoustic flapping");
    assert!(trigger_count <= 50, "Quiet cycles must never trigger false positives");
}

#[test]
fn test_sim_smooth_ambient_noise_adaptation() {
    let mut analyzer = AcousticAnalyzer::new(30.0, 20.0, 48000);

    // Feed a gradual, slow increase in volume over 30 buffers (e.g. ambient HVAC turning on)
    for i in 0..30 {
        let level = 500 + (i * 100);
        let samples = vec![level as i16; 512];
        let triggered = analyzer.ingest_samples(&samples);
        // Gradual increase should adapt the baseline without false triggering
        assert_eq!(triggered, None, "Gradual ambient shifts must adapt baseline and not trigger false alerts");
    }

    assert!(analyzer.baseline_db > 30.0, "Baseline must smoothly track ambient acoustic environment");
}
