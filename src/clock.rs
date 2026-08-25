use std::time::{SystemTime, UNIX_EPOCH};

use crate::constants::MAX_TS;
use crate::error::{Error, Result};

/// Source of millisecond timestamps for ordered generation.
pub trait Clock {
    /// Returns milliseconds since the Unix epoch.
    fn now_ms(&mut self) -> Result<u64>;
}

/// System wall-clock source.
#[derive(Debug, Default, Copy, Clone)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&mut self) -> Result<u64> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Clock("system clock is before the Unix epoch".to_string()))?;
        let millis = duration.as_millis();
        if millis > u128::from(MAX_TS) {
            return Err(Error::Clock(
                "system clock produced timestamp outside 48-bit range".to_string(),
            ));
        }
        Ok(millis as u64)
    }
}

impl<F> Clock for F
where
    F: FnMut() -> u64,
{
    fn now_ms(&mut self) -> Result<u64> {
        Ok(self())
    }
}
