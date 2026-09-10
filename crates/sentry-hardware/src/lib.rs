//! # Sentry Hardware (`sentry-hardware`)
//! Native Linux V4L2 camera capture, ALSA audio monitoring & acoustic alerts.

pub mod audio;
pub mod camera;

pub use audio::SentryAudioSentinel;
pub use camera::SentryCameraSentinel;
