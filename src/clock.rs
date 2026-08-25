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
        #[cfg(all(target_arch = "wasm32", target_os = "unknown", feature = "wasm-js"))]
        {
            let millis = js_sys::Date::now();
            if !millis.is_finite() || millis < 0.0 {
                return Err(Error::Clock("JavaScript clock is invalid".to_string()));
            }
            checked_millis(millis.floor() as u128)
        }
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown", feature = "wasm-js")))]
        {
            let millis = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_millis())
                .map_err(|_| Error::Clock("system clock is before the Unix epoch".to_string()))?;
            checked_millis(millis)
        }
    }
}

fn checked_millis(millis: u128) -> Result<u64> {
    if millis > u128::from(MAX_TS) {
        return Err(Error::Clock(
            "system clock produced timestamp outside 48-bit range".to_string(),
        ));
    }
    Ok(millis as u64)
}

impl<F> Clock for F
where
    F: FnMut() -> u64,
{
    fn now_ms(&mut self) -> Result<u64> {
        Ok(self())
    }
}

#[cfg(test)]
#[path = "clock_tests.rs"]
mod tests;
