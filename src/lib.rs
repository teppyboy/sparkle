//! Sparkle - A reimplementation of Playwright written in Rust, powered by `thirtyfour`.
//!
//! Sparkle provides a high-level API for browser automation, closely matching
//! the Playwright Python API while being idiomatic to Rust.
//!
//! # Features
//! - Full async/await support using Tokio
//! - Type-safe builders for configuration
//! - Comprehensive error handling with Result types
//! - Support for Chromium (Chrome/Edge)
//!
//! # Example
//! ```no_run
//! use sparkle::async_api::Playwright;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let playwright = Playwright::new().await?;
//!     let browser = playwright.chromium().launch(Default::default()).await?;
//!     let page = browser.new_page().await?;
//!     
//!     page.goto("https://example.com", Default::default()).await?;
//!     println!("Title: {}", page.title().await?);
//!     
//!     browser.close().await?;
//!     Ok(())
//! }
//! ```

pub mod async_api;
pub mod cli;
pub mod core;
pub mod driver;

// Re-export commonly used types for convenience
pub use async_api::{Browser, BrowserContext, BrowserType, ElementHandle, ElementInFrame, FrameLocator, Locator, Page, Playwright};
pub use core::{init_logging, init_logging_with_level, Error, Result};

/// Prelude module for convenient imports
///
/// Import everything from this module to get started quickly:
/// ```
/// use sparkle::prelude::*;
/// ```
pub mod prelude {
    pub use crate::async_api::{Browser, BrowserContext, BrowserType, ElementHandle, ElementInFrame, FrameLocator, Locator, Page, Playwright};
    pub use crate::core::{
        init_logging, init_logging_with_level,
        BrowserContextOptions, BrowserContextOptionsBuilder, ClickOptions, ClickOptionsBuilder,
        ConnectOptions, ConnectOptionsBuilder, ConnectOverCdpOptions, ConnectOverCdpOptionsBuilder,
        CookieState, Error, LaunchOptions, LaunchOptionsBuilder, NameValue, NavigationOptions, 
        NavigationOptionsBuilder, OriginState, ProxySettings, Result, SameSite, ScreenshotOptions, 
        ScreenshotOptionsBuilder, StorageState, StorageStateSource, TypeOptions, TypeOptionsBuilder,
        WaitUntilState,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Basic compilation test
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
