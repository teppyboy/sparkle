//! Playwright main entry point
//!
//! This module provides the main Playwright struct which is the entry point
//! for browser automation.

use std::collections::HashMap;

use crate::async_api::browser_type::{BrowserName, BrowserType};
use crate::core::devices::DeviceDescriptor;
use crate::core::Result;

/// Main Playwright instance
///
/// Playwright is the entry point for browser automation. It provides access to
/// different browser types (Chromium, Firefox, WebKit) and device descriptors.
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

    /// Get a device descriptor by name
    ///
    /// Returns `None` if the device name is not found.
    /// Device descriptors are fetched dynamically from Playwright's official repository
    /// on first access.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Playwright;
    /// # async fn example() -> sparkle::core::Result<()> {
    /// let playwright = Playwright::new().await?;
    /// let iphone = playwright.devices("iPhone 12").await?;
    /// if let Some(device) = iphone {
    ///     println!("User agent: {}", device.user_agent);
    ///     // Use device.to_context_options() with browser.new_context()
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn devices(&self, name: &str) -> Result<Option<DeviceDescriptor>> {
        crate::core::devices::get_device(name).await
    }

    /// List all available device names
    ///
    /// Returns a sorted list of all device names available for emulation.
    /// Device descriptors are fetched dynamically from Playwright's official repository
    /// on first access.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Playwright;
    /// # async fn example() -> sparkle::core::Result<()> {
    /// let playwright = Playwright::new().await?;
    /// for device_name in playwright.list_devices().await? {
    ///     println!("{}", device_name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_devices(&self) -> Result<Vec<String>> {
        crate::core::devices::list_devices().await
    }

    /// Get all device descriptors
    ///
    /// Returns a HashMap of all available device descriptors.
    /// Device descriptors are fetched dynamically from Playwright's official repository
    /// on first access.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Playwright;
    /// # async fn example() -> sparkle::core::Result<()> {
    /// let playwright = Playwright::new().await?;
    /// let all_devices = playwright.get_all_devices().await?;
    /// println!("Found {} devices", all_devices.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_all_devices(&self) -> Result<HashMap<String, DeviceDescriptor>> {
        crate::core::devices::get_all_devices().await
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
