//! WebDriver adapter layer
//!
//! This module provides an abstraction over thirtyfour to adapt it to Playwright's
//! semantics and API patterns.

use crate::core::{Error, Result};
use std::sync::Arc;
use std::time::Duration;
use thirtyfour::extensions::cdp::ChromeDevTools;
use thirtyfour::prelude::*;
use tokio::sync::RwLock;

/// Adapter wrapping the thirtyfour WebDriver
///
/// This struct provides a bridge between Playwright's API and thirtyfour's WebDriver,
/// handling conversions and adapting behavior where needed.
pub struct WebDriverAdapter {
    driver: Arc<RwLock<Option<WebDriver>>>,
    slow_mo: Option<Duration>,
    cdp: Arc<RwLock<Option<ChromeDevTools>>>,
}

impl WebDriverAdapter {
    /// Create a new WebDriver adapter from an existing driver
    pub fn new(driver: WebDriver) -> Self {
        let cdp = ChromeDevTools::new(driver.handle.clone());
        Self {
            driver: Arc::new(RwLock::new(Some(driver))),
            slow_mo: None,
            cdp: Arc::new(RwLock::new(Some(cdp))),
        }
    }

    /// Create a new WebDriver adapter with slow_mo
    pub fn new_with_slow_mo(driver: WebDriver, slow_mo: Option<Duration>) -> Self {
        let cdp = ChromeDevTools::new(driver.handle.clone());
        Self {
            driver: Arc::new(RwLock::new(Some(driver))),
            slow_mo,
            cdp: Arc::new(RwLock::new(Some(cdp))),
        }
    }

    /// Apply slow_mo delay before an operation
    async fn apply_slow_mo(&self) {
        if let Some(duration) = self.slow_mo {
            tokio::time::sleep(duration).await;
        }
    }

    /// Create a new WebDriver instance with the given capabilities
    ///
    /// # Arguments
    /// * `url` - WebDriver server URL (e.g., "http://localhost:9515" for ChromeDriver)
    /// * `capabilities` - Browser capabilities as a HashMap
    /// * `slow_mo` - Optional delay to slow down operations
    pub async fn create(
        url: &str, 
        capabilities: std::collections::HashMap<String, serde_json::Value>,
        slow_mo: Option<Duration>,
    ) -> Result<Self> {
        tracing::debug!("Creating WebDriver connection to: {}", url);
        tracing::trace!("Capabilities: {:?}", capabilities);
        
        // Convert HashMap to serde_json::Map
        let caps_map: serde_json::Map<String, serde_json::Value> = 
            capabilities.into_iter().collect();
        let caps: Capabilities = caps_map.into();
        let driver = WebDriver::new(url, caps).await?;
        
        tracing::info!("WebDriver connection established");
        Ok(Self::new_with_slow_mo(driver, slow_mo))
    }

    /// Get a reference to the underlying WebDriver
    ///
    /// Returns an error if the driver has been closed
    pub async fn driver(&self) -> Result<tokio::sync::RwLockReadGuard<'_, Option<WebDriver>>> {
        let guard = self.driver.read().await;
        if guard.is_none() {
            return Err(Error::BrowserClosed);
        }
        Ok(guard)
    }

    /// Get a mutable reference to the underlying WebDriver
    ///
    /// Returns an error if the driver has been closed
    pub async fn driver_mut(&self) -> Result<tokio::sync::RwLockWriteGuard<'_, Option<WebDriver>>> {
        let guard = self.driver.write().await;
        if guard.is_none() {
            return Err(Error::BrowserClosed);
        }
        Ok(guard)
    }

    /// Get a reference to the Chrome DevTools Protocol interface
    ///
    /// Returns an error if the driver has been closed
    pub async fn cdp(&self) -> Result<tokio::sync::RwLockReadGuard<'_, Option<ChromeDevTools>>> {
        let guard = self.cdp.read().await;
        if guard.is_none() {
            return Err(Error::BrowserClosed);
        }
        Ok(guard)
    }

    /// Execute an async closure with the WebDriver
    ///
    /// This is a convenience method to safely access the driver
    pub async fn with_driver<F, T, Fut>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&WebDriver) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        f(driver).await
    }

    /// Navigate to a URL
    pub async fn goto(&self, url: &str) -> Result<()> {
        self.apply_slow_mo().await;
        tracing::debug!("WebDriver: navigating to {}", url);
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        driver.goto(url).await?;
        Ok(())
    }

    /// Get the current URL
    pub async fn current_url(&self) -> Result<String> {
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        let url = driver.current_url().await?;
        Ok(url.to_string())
    }

    /// Get the page title
    pub async fn title(&self) -> Result<String> {
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        let title = driver.title().await?;
        Ok(title)
    }

    /// Find an element by CSS selector
    pub async fn find_element(&self, selector: &str) -> Result<WebElement> {
        self.apply_slow_mo().await;
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        let element = driver
            .find(By::Css(selector))
            .await
            .map_err(|_| Error::element_not_found(selector))?;
        Ok(element)
    }

    /// Find all elements matching a CSS selector
    pub async fn find_elements(&self, selector: &str) -> Result<Vec<WebElement>> {
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        let elements = driver.find_all(By::Css(selector)).await?;
        Ok(elements)
    }

    /// Execute JavaScript in the browser context
    pub async fn execute_script(&self, script: &str) -> Result<serde_json::Value> {
        self.apply_slow_mo().await;
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        let result = driver.execute(script, Vec::new()).await?;
        Ok(result.json().clone())
    }

    /// Execute JavaScript with arguments
    pub async fn execute_script_with_args(
        &self,
        script: &str,
        args: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        let result = driver.execute(script, args).await?;
        Ok(result.json().clone())
    }

    /// Take a screenshot of the current page
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        let screenshot = driver.screenshot_as_png().await?;
        Ok(screenshot)
    }

    /// Close the browser and clean up
    pub async fn close(&self) -> Result<()> {
        tracing::debug!("Closing WebDriver session");
        
        // Clear CDP first
        let mut cdp_guard = self.cdp.write().await;
        cdp_guard.take();
        drop(cdp_guard);
        
        // Then close the driver
        let mut guard = self.driver.write().await;
        if let Some(driver) = guard.take() {
            driver.quit().await?;
            tracing::info!("WebDriver session closed");
        }
        Ok(())
    }

    /// Check if the driver is still active
    pub async fn is_closed(&self) -> bool {
        self.driver.read().await.is_none()
    }

    /// Execute a Chrome DevTools Protocol command
    ///
    /// # Arguments
    /// * `command` - The CDP command to execute (e.g., "Browser.getVersion")
    ///
    /// # Returns
    /// The CDP response as a JSON value
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::driver::WebDriverAdapter;
    /// # async fn example(adapter: &WebDriverAdapter) -> sparkle::core::Result<()> {
    /// let version_info = adapter.execute_cdp("Browser.getVersion").await?;
    /// println!("Browser version: {:?}", version_info);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_cdp(&self, command: &str) -> Result<serde_json::Value> {
        let cdp_guard = self.cdp().await?;
        let dev_tools = cdp_guard.as_ref().ok_or(Error::BrowserClosed)?;
        
        let result = dev_tools.execute_cdp(command).await
            .map_err(|e| Error::ActionFailed(format!("CDP command failed: {}", e)))?;
        
        Ok(result)
    }

    /// Execute a Chrome DevTools Protocol command with parameters
    ///
    /// # Arguments
    /// * `command` - The CDP command to execute
    /// * `params` - Parameters for the CDP command as a JSON value
    ///
    /// # Returns
    /// The CDP response as a JSON value
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::driver::WebDriverAdapter;
    /// # use serde_json::json;
    /// # async fn example(adapter: &WebDriverAdapter) -> sparkle::core::Result<()> {
    /// let params = json!({"expression": "1 + 1"});
    /// let result = adapter.execute_cdp_with_params("Runtime.evaluate", params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_cdp_with_params(
        &self,
        command: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let cdp_guard = self.cdp().await?;
        let dev_tools = cdp_guard.as_ref().ok_or(Error::BrowserClosed)?;
        
        let result = dev_tools.execute_cdp_with_params(command, params).await
            .map_err(|e| Error::ActionFailed(format!("CDP command failed: {}", e)))?;
        
        Ok(result)
    }

    /// Get the browser version
    ///
    /// Returns the browser version string (e.g., "145.0.7632.6")
    /// Uses Chrome DevTools Protocol for accurate version information.
    pub async fn browser_version(&self) -> Result<String> {
        let guard = self.driver().await?;
        let _driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        
        // Try Chrome DevTools Protocol first (works for Chrome/Chromium/Edge)
        // This gives us the most accurate version information
        let cdp_guard = self.cdp().await?;
        let dev_tools = cdp_guard.as_ref().ok_or(Error::BrowserClosed)?;
        
        if let Ok(version_info) = dev_tools.execute_cdp("Browser.getVersion").await {
            // Extract browser version from CDP response
            // The "product" field contains "Chrome/145.0.7632.6" format
            if let Some(product) = version_info.get("product") {
                if let Some(product_str) = product.as_str() {
                    // Extract version number from "Chrome/145.0.7632.6"
                    if let Some(version) = product_str.split('/').nth(1) {
                        return Ok(version.to_string());
                    }
                }
            }
        }
        
        // Fallback: Use JavaScript to get browser version from user agent
        // This works across all browsers (Chrome, Firefox, Safari, Edge, etc.)
        drop(cdp_guard); // Release CDP lock before calling execute_script
        let result = self.execute_script("return navigator.userAgent").await?;
        if let Some(ua) = result.as_str() {
            // Try to extract version from user agent
            // Chrome UA format: "... Chrome/145.0.7632.6 ..."
            // Edge UA format: "... Edg/145.0.7632.6 ..."
            if let Some(start) = ua.find("Chrome/") {
                let version_start = start + 7; // Length of "Chrome/"
                if let Some(end) = ua[version_start..].find(' ') {
                    return Ok(ua[version_start..version_start + end].to_string());
                } else {
                    return Ok(ua[version_start..].to_string());
                }
            } else if let Some(start) = ua.find("Edg/") {
                let version_start = start + 4; // Length of "Edg/"
                if let Some(end) = ua[version_start..].find(' ') {
                    return Ok(ua[version_start..version_start + end].to_string());
                } else {
                    return Ok(ua[version_start..].to_string());
                }
            }
            
            // Fallback to full user agent if we can't parse it
            return Ok(ua.to_string());
        }
        
        // If all else fails, return unknown
        Ok("Unknown".to_string())
    }
}

impl Drop for WebDriverAdapter {
    fn drop(&mut self) {
        // Note: We can't await in Drop, so we just mark it for cleanup
        // The user should call close() explicitly for graceful shutdown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_closed_error() {
        // Create a mock adapter (this would need a real WebDriver in practice)
        // For now, just test that the structure compiles
    }
}
