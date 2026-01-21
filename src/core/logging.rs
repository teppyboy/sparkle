//! Logging initialization for Sparkle
//!
//! This module provides utilities for initializing the tracing subscriber
//! with configuration from environment variables.

use std::sync::Once;
use tracing_subscriber::{fmt, EnvFilter};

static INIT: Once = Once::new();

/// Initialize the tracing subscriber for Sparkle
///
/// This function sets up logging based on the SPARKLE_LOG_LEVEL environment variable.
/// Valid levels: trace, debug, info, warn, error
///
/// If SPARKLE_LOG_LEVEL is not set, logging is disabled by default.
///
/// This function is safe to call multiple times - initialization only happens once.
///
/// # Example
/// ```no_run
/// # use sparkle::core::init_logging;
/// // Set SPARKLE_LOG_LEVEL=debug before running
/// init_logging();
/// ```
pub fn init_logging() {
    INIT.call_once(|| {
        // Check for SPARKLE_LOG_LEVEL environment variable
        let log_level = std::env::var("SPARKLE_LOG_LEVEL").unwrap_or_else(|_| "off".to_string());

        // Only initialize if log level is set to something other than "off"
        if log_level != "off" {
            let filter = EnvFilter::try_from_default_env()
                .or_else(|_| EnvFilter::try_new(&format!("sparkle={}", log_level)))
                .unwrap_or_else(|_| EnvFilter::new("info"));

            fmt()
                .with_env_filter(filter)
                .with_target(true)
                .with_thread_ids(false)
                .with_file(true)
                .with_line_number(true)
                .init();

            tracing::info!("Sparkle logging initialized with level: {}", log_level);
        }
    });
}

/// Initialize logging with a specific level
///
/// This is useful for testing or when you want to override the environment variable.
///
/// # Arguments
/// * `level` - Log level as a string (trace, debug, info, warn, error)
///
/// # Example
/// ```no_run
/// # use sparkle::core::init_logging_with_level;
/// init_logging_with_level("debug");
/// ```
pub fn init_logging_with_level(level: &str) {
    INIT.call_once(|| {
        let filter = EnvFilter::try_new(&format!("sparkle={}", level))
            .unwrap_or_else(|_| EnvFilter::new("info"));

        fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_thread_ids(false)
            .with_file(true)
            .with_line_number(true)
            .init();

        tracing::info!("Sparkle logging initialized with level: {}", level);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging() {
        // Should not panic when called multiple times
        init_logging();
        init_logging();
    }

    #[test]
    fn test_init_logging_with_level() {
        init_logging_with_level("debug");
        // Second call should be ignored
        init_logging_with_level("trace");
    }
}
