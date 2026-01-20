//! ElementHandle API for direct element manipulation
//!
//! ElementHandle represents an in-page DOM element. Unlike Locators which are
//! auto-retrying, ElementHandles point to a specific element at a specific time.

use crate::core::{ClickOptions, Error, Result, TypeOptions};
use thirtyfour::prelude::*;

/// Represents a handle to an in-page DOM element
///
/// ElementHandles are created by querying the page or locator. They point to a
/// specific element in the page and can be used to interact with it.
///
/// # Example
/// ```no_run
/// # use sparkle::async_api::Page;
/// # async fn example(page: &Page) -> sparkle::core::Result<()> {
/// // Note: In practice, you'd use Locators instead of ElementHandles
/// // ElementHandles are lower-level and don't auto-retry
/// # Ok(())
/// # }
/// ```
pub struct ElementHandle {
    element: WebElement,
}

impl ElementHandle {
    /// Create a new ElementHandle from a WebElement
    #[allow(dead_code)]
    pub(crate) fn new(element: WebElement) -> Self {
        Self { element }
    }

    /// Click the element
    ///
    /// # Arguments
    /// * `options` - Click options
    pub async fn click(&self, _options: ClickOptions) -> Result<()> {
        self.element.click().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to click element: {}", e))
        })?;
        Ok(())
    }

    /// Fill the element with text (for input fields)
    ///
    /// This clears the existing value and types the new text.
    ///
    /// # Arguments
    /// * `text` - The text to fill
    pub async fn fill(&self, text: &str) -> Result<()> {
        self.element.clear().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to clear element: {}", e))
        })?;

        self.element.send_keys(text).await.map_err(|e| {
            Error::ActionFailed(format!("Failed to fill element: {}", e))
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
        if let Some(delay) = options.delay {
            // Type with delay between keystrokes
            for ch in text.chars() {
                self.element.send_keys(ch.to_string()).await.map_err(|e| {
                    Error::ActionFailed(format!("Failed to type into element: {}", e))
                })?;
                tokio::time::sleep(delay).await;
            }
        } else {
            // Type all at once
            self.element.send_keys(text).await.map_err(|e| {
                Error::ActionFailed(format!("Failed to type into element: {}", e))
            })?;
        }

        Ok(())
    }

    /// Get the text content of the element
    pub async fn text_content(&self) -> Result<String> {
        let text = self.element.text().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to get text content: {}", e))
        })?;
        Ok(text)
    }

    /// Get the inner text of the element
    pub async fn inner_text(&self) -> Result<String> {
        self.text_content().await
    }

    /// Get an attribute value
    ///
    /// # Arguments
    /// * `name` - The attribute name
    pub async fn get_attribute(&self, name: &str) -> Result<Option<String>> {
        let attr = self.element.attr(name).await.map_err(|e| {
            Error::ActionFailed(format!("Failed to get attribute '{}': {}", name, e))
        })?;
        Ok(attr)
    }

    /// Check if the element is visible
    pub async fn is_visible(&self) -> Result<bool> {
        let visible = self.element.is_displayed().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to check visibility: {}", e))
        })?;
        Ok(visible)
    }

    /// Check if the element is enabled
    pub async fn is_enabled(&self) -> Result<bool> {
        let enabled = self.element.is_enabled().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to check if enabled: {}", e))
        })?;
        Ok(enabled)
    }

    /// Check if a checkbox or radio is checked
    pub async fn is_checked(&self) -> Result<bool> {
        let checked = self.element.is_selected().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to check if checked: {}", e))
        })?;
        Ok(checked)
    }

    /// Get the tag name of the element
    pub async fn tag_name(&self) -> Result<String> {
        let tag = self.element.tag_name().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to get tag name: {}", e))
        })?;
        Ok(tag)
    }

    /// Take a screenshot of the element
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        let screenshot = self.element.screenshot_as_png().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to screenshot element: {}", e))
        })?;
        Ok(screenshot)
    }

    /// Get the bounding box of the element
    ///
    /// Returns (x, y, width, height) in pixels
    pub async fn bounding_box(&self) -> Result<Option<(f64, f64, f64, f64)>> {
        let rect = self.element.rect().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to get bounding box: {}", e))
        })?;
        
        Ok(Some((rect.x, rect.y, rect.width, rect.height)))
    }

    /// Scroll the element into view
    pub async fn scroll_into_view(&self) -> Result<()> {
        self.element
            .scroll_into_view()
            .await
            .map_err(|e| Error::ActionFailed(format!("Failed to scroll into view: {}", e)))?;
        Ok(())
    }

    /// Focus the element
    pub async fn focus(&self) -> Result<()> {
        // WebDriver doesn't have a direct focus method, use JavaScript
        self.element
            .send_keys("")
            .await
            .map_err(|e| Error::ActionFailed(format!("Failed to focus element: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_handle_compiles() {
        // Structure compilation test
    }
}
