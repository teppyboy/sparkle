//! Playwright main entry point
//!
//! This module provides the main Playwright struct which is the entry point
//! for browser automation.

use crate::async_api::browser_type::{BrowserName, BrowserType};
use crate::core::Result;

/// Main Playwright instance
///
/// Playwright is the entry point for browser automation. It provides access to
/// different browser types (Chromium, Firefox, WebKit).
///
/// # Example
/// ```no_run
/// # use sparkle::async_api::Playwright;
/// # async fn example() -> sparkle::core::Result<()> {
/// let playwright = Playwright::new().await?;
/// let browser = playwright.chromium().launch(Default::default()).await?;
/// let page = browser.new_page().await?;
/// page.goto("https://example.com", Default::default()).await?;
/// browser.close().await?;
/// # Ok(())
/// # }
/// ```
pub struct Playwright {
    chromium: BrowserType,
    firefox: BrowserType,
    webkit: BrowserType,
}

impl Playwright {
    /// Create a new Playwright instance
    ///
    /// This is the main entry point for using Sparkle. In Python Playwright,
    /// this is done via context manager. In Rust, we simply create the instance.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Playwright;
    /// # async fn example() -> sparkle::core::Result<()> {
    /// let playwright = Playwright::new().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new() -> Result<Self> {
        Ok(Self {
            chromium: BrowserType::new(BrowserName::Chromium),
            firefox: BrowserType::new(BrowserName::Firefox),
            webkit: BrowserType::new(BrowserName::WebKit),
        })
    }

    /// Get the Chromium browser type
    ///
    /// Use this to launch Chromium or Chrome browsers.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Playwright;
    /// # async fn example() -> sparkle::core::Result<()> {
    /// let playwright = Playwright::new().await?;
    /// let browser = playwright.chromium().launch(Default::default()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn chromium(&self) -> &BrowserType {
        &self.chromium
    }

    /// Get the Firefox browser type
    ///
    /// Use this to launch Firefox browsers.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Playwright;
    /// # async fn example() -> sparkle::core::Result<()> {
    /// let playwright = Playwright::new().await?;
    /// let browser = playwright.firefox().launch(Default::default()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn firefox(&self) -> &BrowserType {
        &self.firefox
    }

    /// Get the WebKit browser type
    ///
    /// Use this to launch WebKit/Safari browsers.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Playwright;
    /// # async fn example() -> sparkle::core::Result<()> {
    /// let playwright = Playwright::new().await?;
    /// let browser = playwright.webkit().launch(Default::default()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn webkit(&self) -> &BrowserType {
        &self.webkit
    }

    /// Stop this Playwright instance
    ///
    /// This is called automatically when the Playwright instance is dropped.
    /// You can call it explicitly if you want to ensure cleanup happens at a
    /// specific time.
    pub fn stop(&mut self) {
        // Cleanup will happen when the instance is dropped
        // For now, this is a no-op but could be extended for cleanup
    }
}

impl Drop for Playwright {
    fn drop(&mut self) {
        // Cleanup code if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_playwright_creation() {
        let playwright = Playwright::new().await.unwrap();
        assert_eq!(playwright.chromium().name(), BrowserName::Chromium);
        assert_eq!(playwright.firefox().name(), BrowserName::Firefox);
        assert_eq!(playwright.webkit().name(), BrowserName::WebKit);
    }
}
