//! Async API for Sparkle (Playwright in Rust)
//!
//! This module provides the async API for browser automation, matching
//! Playwright Python's async_api module.

pub mod browser;
pub mod browser_type;
pub mod element_handle;
pub mod locator;
pub mod playwright;

// Re-export main types
pub use browser::{Browser, BrowserContext, Page};
pub use browser_type::{BrowserName, BrowserType};
pub use element_handle::ElementHandle;
pub use locator::Locator;
pub use playwright::Playwright;