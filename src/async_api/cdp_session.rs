//! Chrome DevTools Protocol (CDP) session
//!
//! This module provides the CDPSession class for low-level Chrome DevTools Protocol access.

use crate::core::Result;
use crate::driver::WebDriverAdapter;
use std::sync::Arc;

/// Represents a Chrome DevTools Protocol session
///
/// CDPSession instances are used to communicate with the Chrome DevTools Protocol.
/// Protocol methods can be called with the `send()` method.
///
/// This matches Playwright's CDPSession API.
///
/// # Example
/// ```no_run
/// # use sparkle::async_api::{Browser, CDPSession};
/// # use serde_json::json;
/// # async fn example(browser: &Browser) -> sparkle::core::Result<()> {
/// let cdp_session = browser.new_browser_cdp_session().await?;
/// 
/// // Get browser version
/// let version = cdp_session.send("Browser.getVersion", None).await?;
/// println!("Version: {:?}", version);
/// 
/// // Evaluate JavaScript
/// let params = json!({"expression": "1 + 1"});
/// let result = cdp_session.send("Runtime.evaluate", Some(params)).await?;
/// println!("Result: {:?}", result);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct CDPSession {
    adapter: Arc<WebDriverAdapter>,
}

impl CDPSession {
    /// Create a new CDP session
    ///
    /// This is typically not called directly; use `Browser::new_browser_cdp_session()` instead.
    pub(crate) fn new(adapter: Arc<WebDriverAdapter>) -> Self {
        Self { adapter }
    }

    /// Send a Chrome DevTools Protocol command
    ///
    /// This is the primary method for interacting with the Chrome DevTools Protocol.
    ///
    /// # Arguments
    /// * `method` - The CDP method to call (e.g., "Browser.getVersion")
    /// * `params` - Optional parameters for the CDP method as a JSON value
    ///
    /// # Returns
    /// The CDP response as a JSON value
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::CDPSession;
    /// # use serde_json::json;
    /// # async fn example(session: &CDPSession) -> sparkle::core::Result<()> {
    /// // Call without parameters
    /// let version = session.send("Browser.getVersion", None).await?;
    /// 
    /// // Call with parameters
    /// let params = json!({"expression": "navigator.userAgent"});
    /// let result = session.send("Runtime.evaluate", Some(params)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        if let Some(params) = params {
            self.adapter.execute_cdp_with_params(method, params).await
        } else {
            self.adapter.execute_cdp(method).await
        }
    }

    /// Detach the CDP session
    ///
    /// Once detached, the CDPSession object can't be used to send messages.
    /// In the current implementation, this is a no-op as the session is tied
    /// to the browser lifetime.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::CDPSession;
    /// # async fn example(session: CDPSession) -> sparkle::core::Result<()> {
    /// session.detach().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detach(&self) -> Result<()> {
        // In our implementation, CDP is tied to the browser lifetime
        // This is a no-op for now, but maintains API compatibility
        Ok(())
    }
}
