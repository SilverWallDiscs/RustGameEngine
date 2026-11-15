#![forbid(unsafe_code)]
//! Core runtime primitives: timing, configuration, events, and tracing.
//!
//! # Examples
//!
//! ~~~rust
//! use engine_core::{EngineConfig, GameLoop};
//! let mut game_loop = GameLoop::new(EngineConfig::default()).unwrap();
//! game_loop.tick(|_fixed_delta| {}, |frame| {
//!     let _seconds = frame.delta_seconds;
//! });
//! ~~~

mod config;
mod events;
mod logging;
mod time;

pub use config::{EngineConfig, EngineConfigError};
pub use events::{EventBus, EventSender};
pub use logging::init_logging;
pub use time::{FrameTime, GameLoop};
