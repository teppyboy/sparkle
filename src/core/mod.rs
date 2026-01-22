//! Core types and utilities for Sparkle

pub mod devices;
pub mod error;
pub mod logging;
pub mod options;
pub mod stealth;
pub mod stealth_headers;
pub mod storage;

// Re-export commonly used types
pub use devices::{get_all_devices, get_device, list_devices, DeviceDescriptor};
pub use error::{Error, Result};
pub use logging::{init_logging, init_logging_with_level};
pub use options::*;
pub use stealth::{get_minimal_stealth_script, get_stealth_script};
pub use stealth_headers::HeadersConfig;
pub use storage::{CookieState, NameValue, OriginState, SameSite, StorageState, StorageStateSource};
