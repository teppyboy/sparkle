//! FrameLocator API for interacting with iframe elements
//!
//!  FrameLocator represents a view into an iframe on the page. It provides methods
//! to interact with elements inside iframes.

use crate::core::{ClickOptions, Error, Result, TypeOptions};
use crate::driver::WebDriverAdapter;
use std::sync::Arc;
use std::time::Duration;

/// Represents a locator scoped to an iframe
///
/// FrameLocator captures the logic to locate an iframe on the page and provides
/// methods to interact with elements within that iframe. It automatically handles
/// switching to the frame context when needed.
///
/// # Example
/// ```no_run
/// # use sparkle::async_api::Page;
/// # async fn example(page: &Page) -> sparkle::core::Result<()> {
/// let frame = page.frame_locator("iframe#my-frame");
/// frame.locator("button#submit").click(Default::default()).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct FrameLocator {
    adapter: Arc<WebDriverAdapter>,
    /// Selector for the iframe element
    frame_selector: String,
    /// Parent frame locator (for nested iframes)
    parent: Option<Box<FrameLocator>>,
    timeout: Duration,
}

impl FrameLocator {
    /// Create a new frame locator
    ///
    /// # Arguments
    /// * `adapter` - WebDriver adapter for browser interaction
    /// * `frame_selector` - CSS selector to locate the iframe element
    pub(crate) fn new(adapter: Arc<WebDriverAdapter>, frame_selector: impl Into<String>) -> Self {
        Self {
            adapter,
            frame_selector: frame_selector.into(),
            parent: None,
            timeout: Duration::from_secs(30),
        }
    }

    /// Create a nested frame locator (child frame within parent frame)
    ///
    /// # Arguments
    /// * `parent` - Parent frame locator
    /// * `frame_selector` - CSS selector for child iframe
    fn new_nested(parent: FrameLocator, frame_selector: impl Into<String>) -> Self {
        let adapter = parent.adapter.clone();
        let timeout = parent.timeout;
        Self {
            adapter,
            frame_selector: frame_selector.into(),
            parent: Some(Box::new(parent)),
            timeout,
        }
    }

    /// Set the timeout for operations within this frame
    ///
    /// # Arguments
    /// * `timeout` - Maximum time to wait for operations
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Get the frame selector string
    pub fn frame_selector(&self) -> &str {
        &self.frame_selector
    }

    /// Switch to this frame's context
    async fn switch_to_frame_context(&self) -> Result<()> {
        // Switch to parent frame first if nested
        if let Some(parent) = &self.parent {
            // Box::pin is needed for recursive async calls
            Box::pin(parent.switch_to_frame_context()).await?;
        } else {
            // If no parent, switch to default content first
            self.adapter.switch_to_default_content().await?;
        }

        // Now switch to this frame using the new selector-based method
        self.adapter.switch_to_frame_by_selector(&self.frame_selector).await?;
        Ok(())
    }

    /// Create a locator for an element within this frame
    ///
    /// Returns a FrameLocator that represents the element within the frame.
    /// Use the returned object's action methods (click, fill, etc.) to interact.
    ///
    /// # Arguments
    /// * `selector` - CSS selector to locate element within the frame
    pub fn locator(&self, selector: impl Into<String>) -> ElementInFrame {
        ElementInFrame {
            frame_locator: self.clone(),
            element_selector: selector.into(),
            timeout: self.timeout,
        }
    }

    /// Create a nested frame locator for an iframe within this frame
    ///
    /// # Arguments
    /// * `selector` - CSS selector for the nested iframe
    pub fn frame_locator(&self, selector: impl Into<String>) -> FrameLocator {
        FrameLocator::new_nested(self.clone(), selector)
    }

    /// Locate element by text content
    pub fn get_by_text(&self, text: impl Into<String>) -> ElementInFrame {
        let text = text.into();
        let selector = format!("//*[text()='{}']", text);
        self.locator(selector)
    }

    /// Locate element by role
    pub fn get_by_role(&self, role: impl Into<String>) -> ElementInFrame {
        let role = role.into();
        let selector = format!("[role='{}']", role);
        self.locator(selector)
    }

    /// Locate element by label text
    pub fn get_by_label(&self, text: impl Into<String>) -> ElementInFrame {
        let text = text.into();
        let selector = format!("label:contains('{}') input, label:contains('{}') textarea", text, text);
        self.locator(selector)
    }

    /// Locate element by placeholder text
    pub fn get_by_placeholder(&self, text: impl Into<String>) -> ElementInFrame {
        let text = text.into();
        let selector = format!("[placeholder='{}']", text);
        self.locator(selector)
    }

    /// Locate element by test ID attribute
    pub fn get_by_test_id(&self, test_id: impl Into<String>) -> ElementInFrame {
        let test_id = test_id.into();
        let selector = format!("[data-testid='{}']", test_id);
        self.locator(selector)
    }
}

/// Represents an element within a frame
///
/// This struct provides methods to interact with elements inside iframes.
/// It handles frame switching automatically.
#[derive(Clone)]
pub struct ElementInFrame {
    frame_locator: FrameLocator,
    element_selector: String,
    timeout: Duration,
}

impl ElementInFrame {
    /// Click the element within the frame
    pub async fn click(&self, _options: ClickOptions) -> Result<()> {
        // Switch to frame context
        self.frame_locator.switch_to_frame_context().await?;
        
        // Find and click element, ensuring we always switch back to default content
        let result = async {
            let element = self.frame_locator.adapter.find_element(&self.element_selector).await?;
            element.click().await?;
            Ok(())
        }.await;
        
        // Always switch back to default content, even if there was an error
        self.frame_locator.adapter.switch_to_default_content().await?;
        
        result
    }

    /// Fill text into an input element within the frame
    pub async fn fill(&self, text: &str) -> Result<()> {
        // Switch to frame context
        self.frame_locator.switch_to_frame_context().await?;
        
        // Find and fill element, ensuring we always switch back
        let result = async {
            let element = self.frame_locator.adapter.find_element(&self.element_selector).await?;
            element.clear().await?;
            element.send_keys(text).await?;
            Ok(())
        }.await;
        
        // Always switch back to default content, even if there was an error
        self.frame_locator.adapter.switch_to_default_content().await?;
        
        result
    }

    /// Type text into an element with delays between keystrokes
    pub async fn r#type(&self, text: &str, _options: TypeOptions) -> Result<()> {
        // For now, just use fill - proper typing with delays can be added later
        self.fill(text).await
    }

    /// Get the text content of the element
    pub async fn text_content(&self) -> Result<Option<String>> {
        // Switch to frame context
        self.frame_locator.switch_to_frame_context().await?;
        
        // Find element and get text, ensuring we always switch back
        let result = async {
            let element = self.frame_locator.adapter.find_element(&self.element_selector).await?;
            let text = element.text().await?;
            Ok(Some(text))
        }.await;
        
        // Always switch back to default content, even if there was an error
        self.frame_locator.adapter.switch_to_default_content().await?;
        
        result
    }

    /// Get inner text of the element
    pub async fn inner_text(&self) -> Result<String> {
        self.text_content().await.map(|t| t.unwrap_or_default())
    }

    /// Get an attribute value
    pub async fn get_attribute(&self, name: &str) -> Result<Option<String>> {
        // Switch to frame context
        self.frame_locator.switch_to_frame_context().await?;
        
        // Find element and get attribute, ensuring we always switch back
        let result = async {
            let element = self.frame_locator.adapter.find_element(&self.element_selector).await?;
            let attr = element.attr(name).await?;
            Ok(attr)
        }.await;
        
        // Always switch back to default content, even if there was an error
        self.frame_locator.adapter.switch_to_default_content().await?;
        
        result
    }

    /// Check if element is visible
    pub async fn is_visible(&self) -> Result<bool> {
        // Switch to frame context
        self.frame_locator.switch_to_frame_context().await?;
        
        // Find element and check visibility, ensuring we always switch back
        let result = async {
            let element = self.frame_locator.adapter.find_element(&self.element_selector).await?;
            let visible = element.is_displayed().await?;
            Ok(visible)
        }.await;
        
        // Always switch back to default content, even if there was an error
        self.frame_locator.adapter.switch_to_default_content().await?;
        
        result
    }

    /// Wait for the element to be visible
    pub async fn wait_for(&self) -> Result<()> {
        let start = std::time::Instant::now();
        
        loop {
            match self.is_visible().await {
                Ok(true) => return Ok(()),
                _ => {
                    if start.elapsed() >= self.timeout {
                        return Err(Error::timeout_duration(
                            &format!("element not visible: {}", self.element_selector),
                            self.timeout,
                        ));
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests require a mock WebDriverAdapter which doesn't exist yet
    // These tests will be skipped for now
    
    #[test]
    fn test_frame_locator_creation() {
        // Test would require mock adapter
    }
}
