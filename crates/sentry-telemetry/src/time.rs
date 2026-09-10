use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NanoTimestamp {
    pub unix_nanos: u128,
}

impl NanoTimestamp {
    pub fn now() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self {
            unix_nanos: duration.as_nanos(),
        }
    }

    pub fn to_rfc3339_nanos(&self) -> String {
        let secs = (self.unix_nanos / 1_000_000_000) as i64;
        let subsec_nanos = (self.unix_nanos % 1_000_000_000) as u32;
        let dt = DateTime::<Utc>::from_timestamp(secs, subsec_nanos).unwrap_or_else(|| Utc::now());
        dt.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)
    }
}

pub struct PicoTimer {
    start: Instant,
}

impl PicoTimer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_nanos(&self) -> u64 {
        self.start.elapsed().as_nanos() as u64
    }

    pub fn elapsed_picos(&self) -> u128 {
        self.start.elapsed().as_nanos() * 1_000
    }
}
