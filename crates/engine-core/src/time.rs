//! Monotonic frame timing and fixed-step orchestration.

use std::time::{Duration, Instant};
use crate::{EngineConfig, EngineConfigError};

/// Timing information for one presentation frame.
///
/// # Examples
///
/// ~~~rust
/// use std::time::Duration;
/// use engine_core::{EngineConfig, GameLoop};
/// let mut game = GameLoop::new(EngineConfig::default()).unwrap();
/// let frame = game.tick_with_delta(Duration::from_millis(16), |_| {}, |_| {});
/// assert_eq!(frame.delta, Duration::from_millis(16));
/// ~~~
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameTime {
    /// Clamped time since the previous frame.
    pub delta: Duration,
    /// Frame delta converted to seconds.
    pub delta_seconds: f32,
    /// Total clamped elapsed time since loop creation.
    pub elapsed: Duration,
    /// Monotonically increasing frame counter, starting at one.
    pub frame_index: u64,
}

/// Loop that decouples fixed simulation updates from variable frame updates.
///
/// A frame executes zero or more fixed updates followed by one variable update. The
/// fixed-update budget prevents an unbounded catch-up spiral after a long stall.
///
/// # Examples
///
/// ~~~rust
/// use engine_core::{EngineConfig, GameLoop};
/// let mut game = GameLoop::new(EngineConfig::default()).unwrap();
/// game.tick(|_fixed_delta| {}, |_frame| {});
/// assert!(game.interpolation_alpha() < 1.0);
/// ~~~
pub struct GameLoop {
    clock: FrameClock,
    fixed: FixedStepper,
}

impl GameLoop {
    /// Creates a loop from validated configuration.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_core::{EngineConfig, GameLoop};
    /// assert!(GameLoop::new(EngineConfig::default()).is_ok());
    /// ~~~
    pub fn new(config: EngineConfig) -> Result<Self, EngineConfigError> {
        config.validate()?;
        let step = Duration::from_secs_f64(1.0 / f64::from(config.fixed_update_hz));
        Ok(Self { clock: FrameClock::new(config.max_frame_delta),
            fixed: FixedStepper::new(step, config.max_fixed_updates_per_frame) })
    }

    /// Measures wall-clock time and runs fixed then variable updates.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_core::{EngineConfig, GameLoop};
    /// let mut game = GameLoop::new(EngineConfig::default()).unwrap();
    /// game.tick(|_| {}, |_| {});
    /// ~~~
    pub fn tick(&mut self, fixed_update: impl FnMut(Duration),
        variable_update: impl FnOnce(FrameTime)) -> FrameTime {
        let frame = self.clock.tick();
        self.run_frame(frame, fixed_update, variable_update)
    }

    /// Runs a frame with explicit time; useful for deterministic tests and replay.
    ///
    /// The configured maximum frame delta is still applied.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use std::time::Duration;
    /// use engine_core::{EngineConfig, GameLoop};
    /// let mut game = GameLoop::new(EngineConfig::default()).unwrap();
    /// let frame = game.tick_with_delta(Duration::from_millis(20), |_| {}, |_| {});
    /// assert_eq!(frame.delta, Duration::from_millis(20));
    /// ~~~
    pub fn tick_with_delta(&mut self, elapsed: Duration, fixed_update: impl FnMut(Duration),
        variable_update: impl FnOnce(FrameTime)) -> FrameTime {
        let frame = self.clock.advance(elapsed);
        self.run_frame(frame, fixed_update, variable_update)
    }

    /// Returns the interpolation fraction between fixed simulation states.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_core::{EngineConfig, GameLoop};
    /// let game = GameLoop::new(EngineConfig::default()).unwrap();
    /// assert!((0.0..1.0).contains(&game.interpolation_alpha()));
    /// ~~~
    pub fn interpolation_alpha(&self) -> f32 {
        self.fixed.alpha()
    }

    /// Returns simulation time discarded after reaching the catch-up budget.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_core::{EngineConfig, GameLoop};
    /// let game = GameLoop::new(EngineConfig::default()).unwrap();
    /// assert!(game.dropped_simulation_time().is_zero());
    /// ~~~
    pub fn dropped_simulation_time(&self) -> Duration {
        self.fixed.dropped
    }

    fn run_frame(&mut self, frame: FrameTime, mut fixed_update: impl FnMut(Duration),
        variable_update: impl FnOnce(FrameTime)) -> FrameTime {
        self.fixed.advance(frame.delta, &mut fixed_update);
        variable_update(frame);
        frame
    }
}

struct FrameClock {
    last: Instant,
    elapsed: Duration,
    index: u64,
    max_delta: Duration,
}
impl FrameClock {
    fn new(max_delta: Duration) -> Self {
        Self { last: Instant::now(), elapsed: Duration::ZERO, index: 0, max_delta }
    }
    fn tick(&mut self) -> FrameTime {
        let now = Instant::now();
        let delta = now.saturating_duration_since(self.last);
        self.last = now;
        self.advance(delta)
    }
    fn advance(&mut self, raw: Duration) -> FrameTime {
        let delta = raw.min(self.max_delta);
        self.elapsed = self.elapsed.saturating_add(delta);
        self.index = self.index.saturating_add(1);
        FrameTime { delta, delta_seconds: delta.as_secs_f32(),
            elapsed: self.elapsed, frame_index: self.index }
    }
}

struct FixedStepper {
    step: Duration,
    accumulator: Duration,
    max_steps: u32,
    dropped: Duration,
}
impl FixedStepper {
    fn new(step: Duration, max_steps: u32) -> Self {
        Self { step, accumulator: Duration::ZERO, max_steps, dropped: Duration::ZERO }
    }
    fn advance(&mut self, delta: Duration, update: &mut impl FnMut(Duration)) {
        self.accumulator = self.accumulator.saturating_add(delta);
        let mut count = 0;
        while self.accumulator >= self.step && count < self.max_steps {
            update(self.step);
            self.accumulator -= self.step;
            count += 1;
        }
        if self.accumulator >= self.step {
            let remainder_ns = self.accumulator.as_nanos() % self.step.as_nanos();
            let remainder = Duration::from_nanos(remainder_ns as u64);
            self.dropped = self.dropped.saturating_add(self.accumulator - remainder);
            self.accumulator = remainder;
        }
    }
    fn alpha(&self) -> f32 {
        (self.accumulator.as_secs_f64() / self.step.as_secs_f64()) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EngineConfig;

    #[test]
    fn frame_delta_is_clamped() {
        let mut clock = FrameClock::new(Duration::from_millis(40));
        let frame = clock.advance(Duration::from_secs(1));
        assert_eq!(frame.delta, Duration::from_millis(40));
        assert_eq!(frame.elapsed, Duration::from_millis(40));
        assert_eq!(frame.frame_index, 1);
    }

    #[test]
    fn catch_up_is_bounded_and_dropped_time_is_accounted() {
        let config = EngineConfig { fixed_update_hz: 10, max_frame_delta: Duration::from_secs(1),
            max_fixed_updates_per_frame: 2 };
        let mut game = GameLoop::new(config).unwrap();
        let mut steps = 0;
        game.tick_with_delta(Duration::from_millis(550), |_| steps += 1, |_| {});
        assert_eq!(steps, 2);
        assert!(game.dropped_simulation_time() >= Duration::from_millis(300));
        assert!((0.0..1.0).contains(&game.interpolation_alpha()));
    }

    #[test]
    fn variable_update_runs_once_without_a_fixed_step() {
        let mut game = GameLoop::new(EngineConfig::default()).unwrap();
        let mut updates = 0;
        game.tick_with_delta(Duration::from_millis(1), |_| panic!("no fixed step due"),
            |_| updates += 1);
        assert_eq!(updates, 1);
    }
}
