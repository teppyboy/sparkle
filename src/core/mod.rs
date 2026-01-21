//! Core types and utilities for Sparkle

pub mod devices;
pub mod error;
pub mod logging;
pub mod options;

// Re-export commonly used types
pub use devices::{get_all_devices, get_device, list_devices, DeviceDescriptor};
pub use error::{Error, Result};
pub use logging::{init_logging, init_logging_with_level};
pub use options::*;
