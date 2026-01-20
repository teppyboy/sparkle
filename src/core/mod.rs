//! Core types and utilities for Sparkle

pub mod error;
pub mod options;

// Re-export commonly used types
pub use error::{Error, Result};
pub use options::*;
