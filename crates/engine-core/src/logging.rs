//! Structured tracing initialization.

use tracing_subscriber::EnvFilter;

/// Installs a formatted tracing subscriber using RUST_LOG when present.
///
/// An already-installed global subscriber is retained.
///
/// # Examples
///
/// ~~~rust
/// use engine_core::init_logging;
/// init_logging();
/// tracing::info!("engine initialized");
/// ~~~
pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt().with_env_filter(filter)
        .with_target(true).try_init();
}
