//! Browser type implementation (Chromium, Firefox, WebKit)
//!
//! This module provides the BrowserType interface for launching browsers.

use crate::async_api::browser::Browser;
use crate::core::{Error, LaunchOptions, Result};
use crate::driver::{ChromeDriverProcess, ChromiumCapabilities, WebDriverAdapter};
use std::path::PathBuf;

/// BrowserType provides methods to launch a specific browser
///
/// This is the entry point for launching browsers. You obtain a BrowserType
/// from the main Playwright instance.
///
/// # Example
/// ```no_run
/// # use sparkle::async_api::Playwright;
/// # async fn example() -> sparkle::core::Result<()> {
/// let playwright = Playwright::new().await?;
/// let chromium = playwright.chromium();
/// let browser = chromium.launch(Default::default()).await?;
/// # Ok(())
/// # }
/// ```
pub struct BrowserType {
    name: BrowserName,
}

/// Browser name enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserName {
    Chromium,
    Firefox,
    WebKit,
}

impl std::fmt::Display for BrowserName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrowserName::Chromium => write!(f, "chromium"),
            BrowserName::Firefox => write!(f, "firefox"),
            BrowserName::WebKit => write!(f, "webkit"),
        }
    }
}

impl BrowserType {
    /// Create a new BrowserType instance
    pub(crate) fn new(name: BrowserName) -> Self {
        Self { name }
    }

    /// Get the browser name
    pub fn name(&self) -> BrowserName {
        self.name
    }

    /// Launch a browser instance
    ///
    /// # Arguments
    /// * `options` - Launch configuration options
    ///
    /// # Returns
    /// A new Browser instance
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::BrowserType;
    /// # use sparkle::core::{LaunchOptions, LaunchOptionsBuilder};
    /// # async fn example(chromium: &BrowserType) -> sparkle::core::Result<()> {
    /// let options = LaunchOptionsBuilder::default()
    ///     .headless(false)
    ///     .build()
    ///     .unwrap();
    /// let browser = chromium.launch(options).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn launch(&self, options: LaunchOptions) -> Result<Browser> {
        match self.name {
            BrowserName::Chromium => self.launch_chromium(options).await,
            BrowserName::Firefox => Err(Error::not_implemented("Firefox support")),
            BrowserName::WebKit => Err(Error::not_implemented("WebKit support")),
        }
    }

    /// Launch Chromium browser
    async fn launch_chromium(&self, options: LaunchOptions) -> Result<Browser> {
        // Build capabilities
        let mut caps = ChromiumCapabilities::new();

        // Set headless mode
        if let Some(headless) = options.headless {
            caps = caps.headless(headless);
        } else {
            caps = caps.headless(true); // Default to headless
        }

        // Add custom arguments
        for arg in &options.args {
            caps = caps.arg(arg.clone());
        }

        // Add common arguments for stability
        caps = caps
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-blink-features=AutomationControlled");

        let capabilities = caps.build();

        // Determine ChromeDriver URL or launch ChromeDriver automatically
        let (chromedriver_url, driver_process) = if let Ok(url) = std::env::var("CHROMEDRIVER_URL") {
            // Use custom ChromeDriver URL from environment variable
            (url, None)
        } else {
            // Check if custom ChromeDriver path is provided via CHROMEDRIVER_PATH
            let driver_path = std::env::var("CHROMEDRIVER_PATH")
                .ok()
                .map(PathBuf::from);
            
            // Launch ChromeDriver automatically from installed location or custom path
            let process = ChromeDriverProcess::launch(driver_path, 9515)
                .await
                .map_err(|e| Error::internal(format!("Failed to launch ChromeDriver: {}", e)))?;
            let url = process.url().to_string();
            (url, Some(process))
        };

        // Create WebDriver adapter
        let adapter = WebDriverAdapter::create(&chromedriver_url, capabilities).await?;

        // Create and return browser with driver process
        Ok(Browser::new(adapter, driver_process))
    }

    /// Get the path to the browser executable
    ///
    /// Returns the path where Playwright expects to find the bundled browser.
    pub fn executable_path(&self) -> String {
        // This would return the actual path to the bundled browser
        // For now, return a placeholder
        match self.name {
            BrowserName::Chromium => "/path/to/chromium".to_string(),
            BrowserName::Firefox => "/path/to/firefox".to_string(),
            BrowserName::WebKit => "/path/to/webkit".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_name_display() {
        assert_eq!(BrowserName::Chromium.to_string(), "chromium");
        assert_eq!(BrowserName::Firefox.to_string(), "firefox");
        assert_eq!(BrowserName::WebKit.to_string(), "webkit");
    }

    #[test]
    fn test_browser_type_creation() {
        let chromium = BrowserType::new(BrowserName::Chromium);
        assert_eq!(chromium.name(), BrowserName::Chromium);
    }
}
