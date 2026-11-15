//! Runtime configuration and validation.

use std::time::Duration;
use thiserror::Error;

/// Configuration for the frame and simulation loops.
///
/// # Examples
///
/// ~~~rust
/// use std::time::Duration;
/// use engine_core::EngineConfig;
/// let config = EngineConfig { fixed_update_hz: 120,
///     max_frame_delta: Duration::from_millis(200),
///     max_fixed_updates_per_frame: 12 };
/// assert!(config.validate().is_ok());
/// ~~~
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineConfig {
    /// Deterministic simulation updates per second.
    pub fixed_update_hz: u32,
    /// Maximum wall-clock time accepted for a rendered frame.
    pub max_frame_delta: Duration,
    /// Maximum fixed updates performed during one rendered frame.
    pub max_fixed_updates_per_frame: u32,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { fixed_update_hz: 60, max_frame_delta: Duration::from_millis(250),
            max_fixed_updates_per_frame: 8 }
    }
}

impl EngineConfig {
    /// Validates that the timing values are usable.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_core::EngineConfig;
    /// assert!(EngineConfig::default().validate().is_ok());
    /// ~~~
    pub fn validate(&self) -> Result<(), EngineConfigError> {
        if self.fixed_update_hz == 0 { return Err(EngineConfigError::ZeroFixedUpdateRate); }
        if self.fixed_update_hz > 1_000_000_000 {
            return Err(EngineConfigError::FixedUpdateRateTooHigh);
        }
        if self.max_frame_delta.is_zero() { return Err(EngineConfigError::ZeroMaxFrameDelta); }
        if self.max_fixed_updates_per_frame == 0 {
            return Err(EngineConfigError::ZeroMaxFixedUpdates);
        }
        Ok(())
    }
}

/// Invalid game-loop configuration.
///
/// # Examples
///
/// ~~~rust
/// use engine_core::{EngineConfig, EngineConfigError};
/// let config = EngineConfig { fixed_update_hz: 0, ..EngineConfig::default() };
/// assert_eq!(config.validate(), Err(EngineConfigError::ZeroFixedUpdateRate));
/// ~~~
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum EngineConfigError {
    /// Fixed simulation rate must be positive.
    #[error("fixed_update_hz must be greater than zero")]
    ZeroFixedUpdateRate,
    /// Fixed update rate exceeds nanosecond timer resolution.
    #[error("fixed_update_hz must not exceed 1,000,000,000")]
    FixedUpdateRateTooHigh,
    /// Frame-delta clamp must be positive.
    #[error("max_frame_delta must be greater than zero")]
    ZeroMaxFrameDelta,
    /// At least one fixed update must be allowed per frame.
    #[error("max_fixed_updates_per_frame must be greater than zero")]
    ZeroMaxFixedUpdates,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_fixed_rate() {
        let config = EngineConfig { fixed_update_hz: 0, ..EngineConfig::default() };
        assert_eq!(config.validate(), Err(EngineConfigError::ZeroFixedUpdateRate));
    }

    #[test]
    fn rejects_rates_beyond_nanosecond_precision() {
        let config = EngineConfig {
            fixed_update_hz: u32::MAX,
            ..EngineConfig::default()
        };
        assert_eq!(config.validate(), Err(EngineConfigError::FixedUpdateRateTooHigh));
    }
}
