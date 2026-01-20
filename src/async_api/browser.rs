//! Browser instance management
//!
//! This module implements the Browser class which represents a browser instance.

use crate::async_api::Locator;
use crate::core::{BrowserContextOptions, ClickOptions, Error, Result, TypeOptions};
use crate::driver::{ChromeDriverProcess, WebDriverAdapter};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents a browser instance
///
/// A Browser is created via `BrowserType::launch()`. It provides methods to
/// create browser contexts and pages.
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
pub struct Browser {
    adapter: Arc<WebDriverAdapter>,
    contexts: Arc<RwLock<Vec<BrowserContext>>>,
    #[allow(dead_code)]
    driver_process: Option<ChromeDriverProcess>,
}

impl Browser {
    /// Create a new Browser instance
    ///
    /// This is typically not called directly; use `BrowserType::launch()` instead.
    pub(crate) fn new(adapter: WebDriverAdapter, driver_process: Option<ChromeDriverProcess>) -> Self {
        Self {
            adapter: Arc::new(adapter),
            contexts: Arc::new(RwLock::new(Vec::new())),
            driver_process,
        }
    }

    /// Create a new browser context
    ///
    /// Browser contexts are isolated environments within a browser instance.
    /// They have their own cookies, cache, and session data.
    ///
    /// # Arguments
    /// * `options` - Configuration options for the context
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Browser;
    /// # use sparkle::core::BrowserContextOptions;
    /// # async fn example(browser: &Browser) -> sparkle::core::Result<()> {
    /// let context = browser.new_context(Default::default()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_context(&self, options: BrowserContextOptions) -> Result<BrowserContext> {
        if self.adapter.is_closed().await {
            return Err(Error::BrowserClosed);
        }

        let context = BrowserContext::new(Arc::clone(&self.adapter), options);
        self.contexts.write().await.push(context.clone());
        Ok(context)
    }

    /// Create a new page in a new browser context
    ///
    /// This is a convenience method that creates a new context and a new page.
    /// Closing this page will close the context as well.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Browser;
    /// # async fn example(browser: &Browser) -> sparkle::core::Result<()> {
    /// let page = browser.new_page().await?;
    /// page.goto("https://example.com", Default::default()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_page(&self) -> Result<Page> {
        let context = self.new_context(Default::default()).await?;
        context.new_page().await
    }

    /// Get all browser contexts
    pub async fn contexts(&self) -> Vec<BrowserContext> {
        self.contexts.read().await.clone()
    }

    /// Close the browser and all of its pages
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Browser;
    /// # async fn example(browser: Browser) -> sparkle::core::Result<()> {
    /// browser.close().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&self) -> Result<()> {
        // Close all contexts
        let contexts = self.contexts.write().await;
        for context in contexts.iter() {
            let _ = context.close().await;
        }
        drop(contexts);

        // Close the browser
        self.adapter.close().await?;
        Ok(())
    }

    /// Check if the browser has been closed
    pub async fn is_closed(&self) -> bool {
        self.adapter.is_closed().await
    }

    /// Get the browser's version
    pub async fn version(&self) -> Result<String> {
        // This would query the browser version through the WebDriver
        // For now, return a placeholder
        Ok("Chromium 120.0.0".to_string())
    }
}

/// Represents an isolated browser context
///
/// Browser contexts are independent environments within a browser instance.
/// They can have different cookies, local storage, and other session data.
#[derive(Clone)]
pub struct BrowserContext {
    adapter: Arc<WebDriverAdapter>,
    _options: BrowserContextOptions,
    pages: Arc<RwLock<Vec<Page>>>,
}

impl BrowserContext {
    /// Create a new browser context
    pub(crate) fn new(adapter: Arc<WebDriverAdapter>, options: BrowserContextOptions) -> Self {
        Self {
            adapter,
            _options: options,
            pages: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new page in this context
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::BrowserContext;
    /// # async fn example(context: &BrowserContext) -> sparkle::core::Result<()> {
    /// let page = context.new_page().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_page(&self) -> Result<Page> {
        if self.adapter.is_closed().await {
            return Err(Error::ContextClosed);
        }

        let page = Page::new(Arc::clone(&self.adapter));
        self.pages.write().await.push(page.clone());
        Ok(page)
    }

    /// Get all pages in this context
    pub async fn pages(&self) -> Vec<Page> {
        self.pages.read().await.clone()
    }

    /// Close the browser context and all its pages
    pub async fn close(&self) -> Result<()> {
        let pages = self.pages.write().await;
        for page in pages.iter() {
            let _ = page.close().await;
        }
        Ok(())
    }
}

/// Represents a single page in a browser context
///
/// Page provides methods to interact with a tab in a browser context.
#[derive(Clone)]
pub struct Page {
    adapter: Arc<WebDriverAdapter>,
    closed: Arc<RwLock<bool>>,
}

impl Page {
    /// Create a new page
    pub(crate) fn new(adapter: Arc<WebDriverAdapter>) -> Self {
        Self {
            adapter,
            closed: Arc::new(RwLock::new(false)),
        }
    }

    /// Navigate to a URL
    ///
    /// # Arguments
    /// * `url` - The URL to navigate to
    /// * `options` - Navigation options (timeout, wait_until, etc.)
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Page;
    /// # use sparkle::core::NavigationOptions;
    /// # async fn example(page: &Page) -> sparkle::core::Result<()> {
    /// page.goto("https://example.com", Default::default()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn goto(
        &self,
        url: &str,
        _options: crate::core::NavigationOptions,
    ) -> Result<()> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.adapter.goto(url).await
    }

    /// Get the current URL
    pub async fn url(&self) -> Result<String> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.adapter.current_url().await
    }

    /// Get the page title
    pub async fn title(&self) -> Result<String> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.adapter.title().await
    }

    /// Take a screenshot of the page
    ///
    /// # Returns
    /// PNG image as bytes
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.adapter.screenshot().await
    }

    /// Close the page
    pub async fn close(&self) -> Result<()> {
        let mut closed = self.closed.write().await;
        if !*closed {
            *closed = true;
            // Page closing is handled at the browser level
        }
        Ok(())
    }

    /// Check if the page is closed
    pub async fn is_closed(&self) -> bool {
        *self.closed.read().await
    }

    /// Create a locator for the given selector
    ///
    /// Locators are the recommended way to interact with elements as they provide
    /// auto-waiting and retry-ability.
    ///
    /// # Arguments
    /// * `selector` - CSS selector to locate the element
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Page;
    /// # async fn example(page: &Page) -> sparkle::core::Result<()> {
    /// let button = page.locator("button#submit");
    /// button.click(Default::default()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn locator(&self, selector: &str) -> Locator {
        Locator::new(Arc::clone(&self.adapter), selector)
    }

    /// Click an element matching the selector
    ///
    /// This is a convenience method equivalent to page.locator(selector).click(options).
    ///
    /// # Arguments
    /// * `selector` - CSS selector to locate the element
    /// * `options` - Click options
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Page;
    /// # async fn example(page: &Page) -> sparkle::core::Result<()> {
    /// page.click("button#submit", Default::default()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn click(&self, selector: &str, options: ClickOptions) -> Result<()> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.locator(selector).click(options).await
    }

    /// Fill an input field with text
    ///
    /// This is a convenience method equivalent to page.locator(selector).fill(text).
    ///
    /// # Arguments
    /// * `selector` - CSS selector to locate the input element
    /// * `text` - Text to fill
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Page;
    /// # async fn example(page: &Page) -> sparkle::core::Result<()> {
    /// page.fill("input[name='email']", "user@example.com").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fill(&self, selector: &str, text: &str) -> Result<()> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.locator(selector).fill(text).await
    }

    /// Type text into an element
    ///
    /// This is a convenience method equivalent to page.locator(selector).type(text, options).
    ///
    /// # Arguments
    /// * `selector` - CSS selector to locate the element
    /// * `text` - Text to type
    /// * `options` - Type options (delay, etc.)
    pub async fn r#type(&self, selector: &str, text: &str, options: TypeOptions) -> Result<()> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.locator(selector).r#type(text, options).await
    }

    /// Get text content of an element
    ///
    /// # Arguments
    /// * `selector` - CSS selector to locate the element
    pub async fn text_content(&self, selector: &str) -> Result<String> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.locator(selector).text_content().await
    }

    /// Check if an element is visible
    ///
    /// # Arguments
    /// * `selector` - CSS selector to locate the element
    pub async fn is_visible(&self, selector: &str) -> Result<bool> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.locator(selector).is_visible().await
    }

    /// Wait for a selector to be visible
    ///
    /// # Arguments
    /// * `selector` - CSS selector to wait for
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Page;
    /// # async fn example(page: &Page) -> sparkle::core::Result<()> {
    /// page.wait_for_selector(".loading").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_selector(&self, selector: &str) -> Result<()> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.locator(selector).wait_for().await
    }

    /// Evaluate JavaScript in the page context
    ///
    /// # Arguments
    /// * `script` - JavaScript code to execute
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Page;
    /// # async fn example(page: &Page) -> sparkle::core::Result<()> {
    /// let result = page.evaluate("document.title").await?;
    /// println!("Result: {:?}", result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn evaluate(&self, script: &str) -> Result<serde_json::Value> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.adapter.execute_script(script).await
    }

    /// Evaluate JavaScript with arguments
    ///
    /// # Arguments
    /// * `script` - JavaScript code to execute
    /// * `args` - Arguments to pass to the script
    pub async fn evaluate_with_args(
        &self,
        script: &str,
        args: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        self.adapter.execute_script_with_args(script, args).await
    }

    /// Get the page content as HTML
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Page;
    /// # async fn example(page: &Page) -> sparkle::core::Result<()> {
    /// let html = page.content().await?;
    /// println!("Page HTML: {}", html);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn content(&self) -> Result<String> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }
        let html = self
            .evaluate("document.documentElement.outerHTML")
            .await?;
        
        // Extract string from JSON value
        if let serde_json::Value::String(s) = html {
            Ok(s)
        } else {
            Ok(html.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_page_closed_error() {
        // This would need a mock WebDriver for proper testing
        // For now, just verify the structure compiles
    }
}
