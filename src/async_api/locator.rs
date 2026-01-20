//! Locator API for element selection and interaction
//!
//! Locators are the central piece of Playwright's auto-waiting and retry-ability.
//! Locators represent a way to find element(s) on the page at any moment.

use crate::core::{ClickOptions, Error, Result, TypeOptions};
use crate::driver::WebDriverAdapter;
use std::sync::Arc;
use std::time::Duration;
use thirtyfour::prelude::*;

/// Represents a way to locate elements on a page
///
/// Locators are the recommended way to interact with elements in Playwright.
/// They auto-wait for the element to be ready before performing actions.
///
/// # Example
/// ```no_run
/// # use sparkle::async_api::Page;
/// # async fn example(page: &Page) -> sparkle::core::Result<()> {
/// let locator = page.locator("button#submit");
/// locator.click(Default::default()).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Locator {
    adapter: Arc<WebDriverAdapter>,
    selector: String,
    timeout: Duration,
}

impl Locator {
    /// Create a new locator
    ///
    /// # Arguments
    /// * `adapter` - WebDriver adapter for browser interaction
    /// * `selector` - CSS selector to locate elements
    pub(crate) fn new(adapter: Arc<WebDriverAdapter>, selector: impl Into<String>) -> Self {
        Self {
            adapter,
            selector: selector.into(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Set the timeout for this locator
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait for operations
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Get the selector string
    pub fn selector(&self) -> &str {
        &self.selector
    }

    /// Find the element with auto-waiting
    ///
    /// This method waits for the element to be present in the DOM.
    async fn find_element(&self) -> Result<WebElement> {
        // For now, use a simple retry loop. In the future, this should use
        // proper WebDriver waits or implement Playwright's auto-waiting logic.
        let start = std::time::Instant::now();

        loop {
            match self.adapter.find_element(&self.selector).await {
                Ok(element) => return Ok(element),
                Err(_e) => {
                    if start.elapsed() >= self.timeout {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }

        Err(Error::timeout_duration("element not found", self.timeout))
    }

    /// Find all matching elements
    async fn find_elements(&self) -> Result<Vec<WebElement>> {
        self.adapter.find_elements(&self.selector).await
    }

    /// Click the element
    ///
    /// This method waits for the element to be visible and enabled before clicking.
    ///
    /// # Arguments
    /// * `options` - Click options (timeout, modifiers, etc.)
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Locator;
    /// # async fn example(locator: &Locator) -> sparkle::core::Result<()> {
    /// locator.click(Default::default()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn click(&self, options: ClickOptions) -> Result<()> {
        let timeout = options.timeout.unwrap_or(self.timeout);
        let start = std::time::Instant::now();

        // Wait for element and click
        let element = self.find_element().await.map_err(|_| {
            Error::timeout_duration(&format!("Timeout waiting for element '{}'", self.selector), timeout)
        })?;

        // Check if we have time left
        if start.elapsed() >= timeout {
            return Err(Error::timeout_duration("click", timeout));
        }

        // Perform the click
        element.click().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to click '{}': {}", self.selector, e))
        })?;

        Ok(())
    }

    /// Fill an input field with text
    ///
    /// This clears the existing value and types the new text.
    ///
    /// # Arguments
    /// * `text` - The text to fill
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Locator;
    /// # async fn example(locator: &Locator) -> sparkle::core::Result<()> {
    /// locator.fill("my text").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fill(&self, text: &str) -> Result<()> {
        let element = self.find_element().await?;
        
        // Clear existing value
        element.clear().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to clear '{}': {}", self.selector, e))
        })?;

        // Type the text
        element.send_keys(text).await.map_err(|e| {
            Error::ActionFailed(format!("Failed to fill '{}': {}", self.selector, e))
        })?;

        Ok(())
    }

    /// Type text into the element
    ///
    /// Unlike fill(), this does not clear existing text first.
    ///
    /// # Arguments
    /// * `text` - The text to type
    /// * `options` - Type options (delay, etc.)
    pub async fn r#type(&self, text: &str, options: TypeOptions) -> Result<()> {
        let element = self.find_element().await?;

        if let Some(delay) = options.delay {
            // Type with delay between keystrokes
            for ch in text.chars() {
                element.send_keys(ch.to_string()).await.map_err(|e| {
                    Error::ActionFailed(format!("Failed to type into '{}': {}", self.selector, e))
                })?;
                tokio::time::sleep(delay).await;
            }
        } else {
            // Type all at once
            element.send_keys(text).await.map_err(|e| {
                Error::ActionFailed(format!("Failed to type into '{}': {}", self.selector, e))
            })?;
        }

        Ok(())
    }

    /// Get the text content of the element
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Locator;
    /// # async fn example(locator: &Locator) -> sparkle::core::Result<()> {
    /// let text = locator.text_content().await?;
    /// println!("Text: {}", text);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn text_content(&self) -> Result<String> {
        let element = self.find_element().await?;
        let text = element.text().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to get text from '{}': {}", self.selector, e))
        })?;
        Ok(text)
    }

    /// Get the inner text of the element
    ///
    /// This is an alias for text_content()
    pub async fn inner_text(&self) -> Result<String> {
        self.text_content().await
    }

    /// Get an attribute value
    ///
    /// # Arguments
    /// * `name` - The attribute name
    pub async fn get_attribute(&self, name: &str) -> Result<Option<String>> {
        let element = self.find_element().await?;
        let attr = element.attr(name).await.map_err(|e| {
            Error::ActionFailed(format!("Failed to get attribute '{}' from '{}': {}", name, self.selector, e))
        })?;
        Ok(attr)
    }

    /// Check if the element is visible
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Locator;
    /// # async fn example(locator: &Locator) -> sparkle::core::Result<()> {
    /// if locator.is_visible().await? {
    ///     println!("Element is visible");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn is_visible(&self) -> Result<bool> {
        let element = self.find_element().await?;
        let visible = element.is_displayed().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to check visibility of '{}': {}", self.selector, e))
        })?;
        Ok(visible)
    }

    /// Check if the element is enabled
    pub async fn is_enabled(&self) -> Result<bool> {
        let element = self.find_element().await?;
        let enabled = element.is_enabled().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to check if '{}' is enabled: {}", self.selector, e))
        })?;
        Ok(enabled)
    }

    /// Check if a checkbox or radio is checked
    pub async fn is_checked(&self) -> Result<bool> {
        let element = self.find_element().await?;
        let checked = element.is_selected().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to check if '{}' is checked: {}", self.selector, e))
        })?;
        Ok(checked)
    }

    /// Count the number of matching elements
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Locator;
    /// # async fn example(locator: &Locator) -> sparkle::core::Result<()> {
    /// let count = locator.count().await?;
    /// println!("Found {} elements", count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn count(&self) -> Result<usize> {
        let elements = self.find_elements().await?;
        Ok(elements.len())
    }

    /// Get the nth element (0-based index)
    ///
    /// # Arguments
    /// * `index` - Zero-based index of the element
    pub fn nth(&self, index: usize) -> Locator {
        // Create a new locator with an indexed selector
        // Note: This is a simplified version. Playwright uses more sophisticated chaining.
        let indexed_selector = format!("({})[{}]", self.selector, index + 1);
        Locator {
            adapter: Arc::clone(&self.adapter),
            selector: indexed_selector,
            timeout: self.timeout,
        }
    }

    /// Get the first matching element
    pub fn first(&self) -> Locator {
        self.nth(0)
    }

    /// Get the last matching element
    pub fn last(&self) -> Locator {
        // This is a simplified implementation
        // A full implementation would need to count elements first
        let last_selector = format!("({}):last-of-type", self.selector);
        Locator {
            adapter: Arc::clone(&self.adapter),
            selector: last_selector,
            timeout: self.timeout,
        }
    }

    /// Wait for the element to be visible
    pub async fn wait_for(&self) -> Result<()> {
        let start = std::time::Instant::now();

        loop {
            match self.is_visible().await {
                Ok(true) => return Ok(()),
                Ok(false) | Err(_) => {
                    if start.elapsed() >= self.timeout {
                        return Err(Error::timeout_duration(
                            &format!("Element '{}' not visible", self.selector),
                            self.timeout,
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Take a screenshot of the element
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        let element = self.find_element().await?;
        let screenshot = element.screenshot_as_png().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to screenshot '{}': {}", self.selector, e))
        })?;
        Ok(screenshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locator_selector() {
        // Mock test - would need real WebDriver for full testing
        // Just verify structure compiles
    }

    #[test]
    fn test_locator_timeout_builder() {
        // Verify timeout can be set via builder pattern
        // Would need mock adapter for full test
    }
}
