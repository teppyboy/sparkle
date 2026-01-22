//! Browser instance management
//!
//! This module implements the Browser class which represents a browser instance.

use crate::async_api::Locator;
use crate::async_api::CDPSession;
use crate::core::{BrowserContextOptions, ClickOptions, Error, Result, TypeOptions};
use crate::driver::{ChromeDriverProcess, WebDriverAdapter};
use std::sync::Arc;
use std::time::Duration;
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
    stealth_options: Option<crate::core::StealthOptions>,
}

impl Browser {
    /// Create a new Browser instance
    ///
    /// This is typically not called directly; use `BrowserType::launch()` instead.
    pub(crate) fn new(
        adapter: WebDriverAdapter,
        driver_process: Option<ChromeDriverProcess>,
        stealth_options: Option<crate::core::StealthOptions>,
    ) -> Self {
        Self {
            adapter: Arc::new(adapter),
            contexts: Arc::new(RwLock::new(Vec::new())),
            driver_process,
            stealth_options,
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
        tracing::debug!("Creating new browser context");
        
        if self.adapter.is_closed().await {
            tracing::error!("Cannot create context: browser is closed");
            return Err(Error::BrowserClosed);
        }

        // Load storage state if provided
        let storage_state = if let Some(source) = options.storage_state.clone() {
            Some(source.load()?)
        } else {
            None
        };

        let context = BrowserContext::new(Arc::clone(&self.adapter), options);
        
        // Apply storage state if loaded
        if let Some(state) = storage_state {
            tracing::debug!("Applying storage state to context");
            context.apply_storage_state(&state).await?;
        }
        
        self.contexts.write().await.push(context.clone());
        
        tracing::info!("Browser context created successfully");
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
        tracing::debug!("Creating new page");
        
        // Create context with stealth options from browser
        let mut context_options = BrowserContextOptions::default();
        context_options.stealth = self.stealth_options.clone();
        
        let context = self.new_context(context_options).await?;
        let page = context.new_page().await?;
        tracing::info!("Page created successfully");
        Ok(page)
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
        tracing::info!("Closing browser");
        
        // Close all contexts
        let contexts = self.contexts.write().await;
        tracing::debug!("Closing {} browser contexts", contexts.len());
        for context in contexts.iter() {
            let _ = context.close().await;
        }
        drop(contexts);

        // Close the browser
        self.adapter.close().await?;
        tracing::info!("Browser closed successfully");
        Ok(())
    }

    /// Check if the browser has been closed
    pub async fn is_closed(&self) -> bool {
        self.adapter.is_closed().await
    }

    /// Get the browser's version
    ///
    /// Returns the browser version string (e.g., "145.0.7632.6")
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Browser;
    /// # async fn example(browser: &Browser) -> sparkle::core::Result<()> {
    /// let version = browser.version().await?;
    /// println!("Browser version: {}", version);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn version(&self) -> Result<String> {
        self.adapter.browser_version().await
    }

    /// Create a new Chrome DevTools Protocol session
    ///
    /// Returns a CDPSession object that can be used to send CDP commands.
    /// This matches Playwright API.
    ///
    /// Note: CDP sessions are only supported on Chromium-based browsers.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Browser;
    /// # use serde_json::json;
    /// # async fn example(browser: &Browser) -> sparkle::core::Result<()> {
    /// let cdp_session = browser.new_browser_cdp_session().await?;
    /// 
    /// // Get browser version
    /// let version = cdp_session.send("Browser.getVersion", None).await?;
    /// 
    /// // Evaluate JavaScript
    /// let params = json!({"expression": "1 + 1"});
    /// let result = cdp_session.send("Runtime.evaluate", Some(params)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_browser_cdp_session(&self) -> Result<CDPSession> {
        if self.adapter.is_closed().await {
            return Err(Error::BrowserClosed);
        }
        Ok(CDPSession::new(Arc::clone(&self.adapter)))
    }

    /// Execute a Chrome DevTools Protocol command (Sparkle Extension)
    ///
    /// **IMPORTANT**: This is a Sparkle-specific convenience method that does NOT exist
    /// in Playwright. For Playwright compatibility, use browser.new_browser_cdp_session() instead.
    ///
    /// # Arguments
    /// * `command` - The CDP command to execute
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Browser;
    /// # async fn example(browser: &Browser) -> sparkle::core::Result<()> {
    /// // Sparkle convenience (not in Playwright)
    /// let info = browser.execute_cdp("Browser.getVersion").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_cdp(&self, command: &str) -> Result<serde_json::Value> {
        self.adapter.execute_cdp(command).await
    }

    /// Execute a Chrome DevTools Protocol command with parameters (Sparkle Extension)
    ///
    /// **IMPORTANT**: This is a Sparkle-specific convenience method that does NOT exist
    /// in Playwright. For Playwright compatibility, use browser.new_browser_cdp_session() instead.
    ///
    /// # Arguments
    /// * `command` - The CDP command to execute
    /// * `params` - Parameters for the CDP command
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Browser;
    /// # use serde_json::json;
    /// # async fn example(browser: &Browser) -> sparkle::core::Result<()> {
    /// // Sparkle convenience (not in Playwright)
    /// let params = json!({"expression": "1 + 1"});
    /// let result = browser.execute_cdp_with_params("Runtime.evaluate", params).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_cdp_with_params(
        &self,
        command: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.adapter.execute_cdp_with_params(command, params).await
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
    stealth_options: Option<crate::core::StealthOptions>,
}

impl BrowserContext {
    /// Create a new browser context
    pub(crate) fn new(adapter: Arc<WebDriverAdapter>, options: BrowserContextOptions) -> Self {
        let stealth_options = options.stealth.clone();
        Self {
            adapter,
            _options: options,
            pages: Arc::new(RwLock::new(Vec::new())),
            stealth_options,
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

        let page = Page::new(Arc::clone(&self.adapter), self.stealth_options.clone()).await?;
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

    /// Get the current storage state (cookies, localStorage, sessionStorage)
    ///
    /// This matches Playwright's storage_state() API.
    ///
    /// # Arguments
    /// * `path` - Optional file path to save the storage state as JSON
    ///
    /// # Returns
    /// The storage state containing cookies and origin storage
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::BrowserContext;
    /// # async fn example(context: &BrowserContext) -> sparkle::core::Result<()> {
    /// // Save to file
    /// let state = context.storage_state(Some("auth.json")).await?;
    ///
    /// // Or just get the state without saving
    /// let state = context.storage_state(None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn storage_state(&self, path: Option<impl Into<std::path::PathBuf>>) -> Result<crate::core::StorageState> {
        use crate::core::storage::{OriginState, StorageState};
        use std::collections::HashMap;
        
        tracing::debug!("Getting storage state for context");
        
        if self.adapter.is_closed().await {
            return Err(Error::ContextClosed);
        }

        // Get all cookies
        let cookies = self.adapter.get_cookies().await?;
        tracing::debug!("Retrieved {} cookies", cookies.len());

        // Get storage from all open pages
        let pages = self.pages.read().await;
        let mut origins_map: HashMap<String, OriginState> = HashMap::new();

        for page in pages.iter() {
            if page.is_closed().await {
                continue;
            }

            // Get the page's origin
            let url = match page.url().await {
                Ok(u) => u,
                Err(_) => continue,
            };

            // Parse origin from URL
            let origin = if let Ok(parsed_url) = url::Url::parse(&url) {
                if let Some(host) = parsed_url.host_str() {
                    let scheme = parsed_url.scheme();
                    let port = parsed_url.port()
                        .map(|p| format!(":{}", p))
                        .unwrap_or_default();
                    format!("{}://{}{}", scheme, host, port)
                } else {
                    continue;
                }
            } else {
                continue;
            };

            // Skip if we already have this origin
            if origins_map.contains_key(&origin) {
                continue;
            }

            // Get storage for this origin
            match self.adapter.get_storage_for_origin(&origin).await {
                Ok((local_storage, session_storage)) => {
                    origins_map.insert(
                        origin.clone(),
                        OriginState {
                            origin,
                            local_storage,
                            session_storage,
                        },
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to get storage for origin '{}': {}", origin, e);
                }
            }
        }

        let origins: Vec<OriginState> = origins_map.into_values().collect();
        tracing::debug!("Retrieved storage for {} origins", origins.len());

        let state = StorageState { cookies, origins };

        // Save to file if path provided
        if let Some(p) = path {
            let path_buf = p.into();
            state.to_file(&path_buf)?;
            tracing::info!("Storage state saved to: {}", path_buf.display());
        }

        Ok(state)
    }

    /// Apply storage state to the context (internal helper)
    ///
    /// Sets cookies and storage from a StorageState object.
    /// This is called during context creation when storage_state option is provided.
    pub(crate) async fn apply_storage_state(&self, state: &crate::core::StorageState) -> Result<()> {
        tracing::debug!("Applying storage state: {} cookies, {} origins", 
            state.cookies.len(), state.origins.len());
        
        if self.adapter.is_closed().await {
            return Err(Error::ContextClosed);
        }

        // Set cookies first
        if !state.cookies.is_empty() {
            self.adapter.set_cookies(&state.cookies).await?;
            tracing::debug!("Applied {} cookies", state.cookies.len());
        }

        // Set storage for each origin
        for origin_state in &state.origins {
            if origin_state.local_storage.is_empty() && origin_state.session_storage.is_empty() {
                continue;
            }

            tracing::debug!("Setting storage for origin: {}", origin_state.origin);

            // Create a temporary page and navigate to the origin to set storage
            let page = self.new_page().await?;
            
            // Navigate to the origin
            if let Err(e) = page.goto(&origin_state.origin, Default::default()).await {
                tracing::warn!("Failed to navigate to origin '{}': {}", origin_state.origin, e);
                let _ = page.close().await;
                continue;
            }

            // Set storage
            if let Err(e) = self.adapter.set_storage(
                &origin_state.local_storage,
                &origin_state.session_storage
            ).await {
                tracing::warn!("Failed to set storage for origin '{}': {}", origin_state.origin, e);
            }

            // Close the temporary page
            let _ = page.close().await;
            
            // Remove from pages list
            let mut pages = self.pages.write().await;
            pages.retain(|p| !std::ptr::eq(p as *const _, &page as *const _));
        }

        tracing::info!("Storage state applied successfully");
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
    pub(crate) async fn new(
        adapter: Arc<WebDriverAdapter>,
        stealth_options: Option<crate::core::StealthOptions>,
    ) -> Result<Self> {
        let page = Self {
            adapter,
            closed: Arc::new(RwLock::new(false)),
        };
        
        // Inject stealth script if stealth is enabled
        if let Some(stealth_opts) = stealth_options {
            if stealth_opts.enabled {
                page.inject_stealth_features(&stealth_opts).await?;
            }
        }
        
        Ok(page)
    }
    
    /// Inject all stealth features via CDP
    async fn inject_stealth_features(&self, stealth_options: &crate::core::StealthOptions) -> Result<()> {
        use serde_json::json;
        
        // 1. Set User-Agent and headers via CDP if header_alignment is enabled
        if stealth_options.header_alignment {
            // Get browser version
            let version = self.adapter.browser_version().await.unwrap_or_else(|_| "120.0.0.0".to_string());
            
            // Generate headers configuration
            let headers_config = crate::core::stealth_headers::HeadersConfig::from_stealth_options(
                stealth_options,
                &version,
            );
            
            // Use CDP Network.setUserAgentOverride
            let params = json!({
                "userAgent": headers_config.user_agent,
                "acceptLanguage": headers_config.accept_language,
                "platform": headers_config.platform,
            });
            
            self.adapter.execute_cdp_with_params("Network.setUserAgentOverride", params)
                .await
                .map_err(|e| Error::ActionFailed(format!("Failed to set user agent: {}", e)))?;
            
            tracing::debug!("User-Agent and headers set successfully");
        }
        
        // 2. Set timezone if specified
        if let Some(ref timezone_id) = stealth_options.timezone_id {
            let params = json!({
                "timezoneId": timezone_id,
            });
            
            self.adapter.execute_cdp_with_params("Emulation.setTimezoneOverride", params)
                .await
                .map_err(|e| Error::ActionFailed(format!("Failed to set timezone: {}", e)))?;
            
            tracing::debug!("Timezone set to: {}", timezone_id);
        }
        
        // 3. Set locale if specified
        if let Some(ref locale) = stealth_options.locale {
            let params = json!({
                "locale": locale,
            });
            
            self.adapter.execute_cdp_with_params("Emulation.setLocaleOverride", params)
                .await
                .ok(); // Ignore error - this command might not be supported in all versions
            
            tracing::debug!("Locale set to: {}", locale);
        }
        
        // 4. Set geolocation if specified
        if let Some((latitude, longitude, accuracy)) = stealth_options.geolocation {
            let params = json!({
                "latitude": latitude,
                "longitude": longitude,
                "accuracy": accuracy,
            });
            
            self.adapter.execute_cdp_with_params("Emulation.setGeolocationOverride", params)
                .await
                .map_err(|e| Error::ActionFailed(format!("Failed to set geolocation: {}", e)))?;
            
            tracing::debug!("Geolocation set to: {}, {}", latitude, longitude);
        }
        
        // 5. Inject stealth JavaScript
        let script = crate::core::stealth::get_stealth_script(
            stealth_options.webgl_spoof,
            stealth_options.canvas_noise,
            stealth_options.permissions_patch,
        );
        
        // Use CDP Page.addScriptToEvaluateOnNewDocument to inject on every frame/page load
        let params = json!({
            "source": script,
            "runImmediately": true
        });
        
        self.adapter.execute_cdp_with_params("Page.addScriptToEvaluateOnNewDocument", params)
            .await
            .map_err(|e| Error::ActionFailed(format!("Failed to inject stealth script: {}", e)))?;
        
        tracing::debug!("Stealth features injected successfully");
        Ok(())
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
        tracing::info!("Navigating to: {}", url);
        
        if *self.closed.read().await {
            tracing::error!("Cannot navigate: page is closed");
            return Err(Error::PageClosed);
        }
        
        self.adapter.goto(url).await?;
        tracing::debug!("Navigation completed successfully");
        Ok(())
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
        tracing::debug!("Clicking element: {}", selector);
        
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

    /// Wait for the page to reach a specific load state
    ///
    /// Returns when the required load state has been reached. This resolves immediately
    /// if the load state is already reached.
    ///
    /// # Arguments
    /// * `state` - Optional load state to wait for. Defaults to `Load`.
    ///   - `Load` - wait for the `load` event (page fully loaded)
    ///   - `DomContentLoaded` - wait for `DOMContentLoaded` event
    ///   - `NetworkIdle` - wait until there are no network connections for at least 500ms
    ///   - `Commit` - wait for navigation to be committed
    /// * `timeout` - Optional timeout duration. Defaults to 30 seconds.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Page;
    /// # use sparkle::core::WaitUntilState;
    /// # use std::time::Duration;
    /// # async fn example(page: &Page) -> sparkle::core::Result<()> {
    /// // Wait for full page load
    /// page.wait_for_load_state(None, None).await?;
    ///
    /// // Wait for DOM content loaded
    /// page.wait_for_load_state(Some(WaitUntilState::DomContentLoaded), None).await?;
    ///
    /// // Wait for network idle with custom timeout
    /// page.wait_for_load_state(
    ///     Some(WaitUntilState::NetworkIdle),
    ///     Some(Duration::from_secs(10))
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn wait_for_load_state(
        &self,
        state: Option<crate::core::WaitUntilState>,
        timeout: Option<Duration>,
    ) -> Result<()> {
        if *self.closed.read().await {
            return Err(Error::PageClosed);
        }

        let load_state = state.unwrap_or(crate::core::WaitUntilState::Load);
        let timeout_duration = timeout.unwrap_or(Duration::from_secs(30));

        tracing::debug!("Page: waiting for load state {:?}", load_state);
        self.adapter.wait_for_load_state(load_state, timeout_duration).await
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
