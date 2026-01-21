//! Core types and utilities for Sparkle

pub mod error;
pub mod logging;
pub mod options;

// Re-export commonly used types
pub use error::{Error, Result};
pub use logging::{init_logging, init_logging_with_level};
pub use options::*;
