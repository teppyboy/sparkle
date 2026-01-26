//! WebDriver adapter layer
//!
//! This module provides an abstraction over thirtyfour to adapt it to Playwright's
//! semantics and API patterns.

use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use futures::{SinkExt, StreamExt};
use http::Method;
use reqwest::Client;
use serde_json::{json, Value};
use thirtyfour::common::command::{Command, ExtensionCommand};
use thirtyfour::error::WebDriverErrorInner;
use thirtyfour::extensions::cdp::ChromeDevTools;
use thirtyfour::prelude::*;
use tokio::sync::RwLock;
use tokio::time::{Instant, Sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use crate::core::{Error, Result};

/// Adapter wrapping the thirtyfour WebDriver
///
/// This struct provides a bridge between Playwright's API and thirtyfour's WebDriver,
/// handling conversions and adapting behavior where needed.
pub struct WebDriverAdapter {
    driver: Arc<RwLock<Option<WebDriver>>>,
    slow_mo: Option<Duration>,
    cdp: Arc<RwLock<Option<ChromeDevTools>>>,
    requested_capabilities: Option<serde_json::Map<String, serde_json::Value>>,
    session_capabilities: Arc<RwLock<Option<serde_json::Value>>>,
}

#[derive(Clone, Debug, Default)]
struct LoadStateSnapshot {
    domcontentloaded: bool,
    load: bool,
    network_idle: bool,
    commit: bool,
}

const W3C_ELEMENT_KEY: &str = "element-6066-11e4-a52e-4f735466cecf";
const W3C_SHADOW_KEY: &str = "shadow-6066-11e4-a52e-4f735466cecf";
const LEGACY_ELEMENT_KEY: &str = "ELEMENT";


#[derive(Debug)]
struct GetSessionCommand;

impl ExtensionCommand for GetSessionCommand {
    fn parameters_json(&self) -> Option<Value> {
        None
    }

    fn method(&self) -> Method {
        Method::GET
    }

    fn endpoint(&self) -> Arc<str> {
        Arc::from("")
    }
}

impl WebDriverAdapter {
    /// Create a new WebDriver adapter from an existing driver
    pub fn new(driver: WebDriver) -> Self {
        let cdp = ChromeDevTools::new(driver.handle.clone());
        Self {
            driver: Arc::new(RwLock::new(Some(driver))),
            slow_mo: None,
            cdp: Arc::new(RwLock::new(Some(cdp))),
            requested_capabilities: None,
            session_capabilities: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new WebDriver adapter with slow_mo
    pub fn new_with_slow_mo(driver: WebDriver, slow_mo: Option<Duration>) -> Self {
        let cdp = ChromeDevTools::new(driver.handle.clone());
        Self {
            driver: Arc::new(RwLock::new(Some(driver))),
            slow_mo,
            cdp: Arc::new(RwLock::new(Some(cdp))),
            requested_capabilities: None,
            session_capabilities: Arc::new(RwLock::new(None)),
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
        let caps: Capabilities = caps_map.clone().into();
        let driver = WebDriver::new(url, caps).await?;
        let cdp = ChromeDevTools::new(driver.handle.clone());
        
        tracing::info!("WebDriver connection established");
        Ok(Self {
            driver: Arc::new(RwLock::new(Some(driver))),
            slow_mo,
            cdp: Arc::new(RwLock::new(Some(cdp))),
            requested_capabilities: Some(caps_map),
            session_capabilities: Arc::new(RwLock::new(None)),
        })
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

    async fn session_capabilities(&self) -> Result<Option<serde_json::Value>> {
        let mut guard = self.session_capabilities.write().await;
        if let Some(cached) = guard.as_ref() {
            return Ok(Some(cached.clone()));
        }

        let driver_guard = self.driver().await?;
        let driver = driver_guard.as_ref().ok_or(Error::BrowserClosed)?;

        let response = match driver
            .handle
            .cmd(Command::ExtensionCommand(Box::new(GetSessionCommand)))
            .await
        {
            Ok(response) => response,
            Err(error) => {
                tracing::debug!("GetSession not available via WebDriver: {}", error);
                return Ok(None);
            }
        };

        let value: serde_json::Value = match response.value() {
            Ok(value) => value,
            Err(error) => {
                tracing::debug!("Failed to parse session response: {}", error);
                return Ok(None);
            }
        };

        let capabilities = value
            .get("capabilities")
            .cloned()
            .or_else(|| value.get("capability").cloned());

        if let Some(capabilities) = capabilities.clone() {
            *guard = Some(capabilities.clone());
            return Ok(Some(capabilities));
        }

        *guard = Some(serde_json::Value::Null);
        Ok(None)
    }

    fn extract_browser_version(capabilities: &serde_json::Value) -> Option<String> {
        capabilities
            .get("browserVersion")
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .or_else(|| {
                capabilities
                    .get("browser_version")
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
            })
    }

    fn origin_from_url(url: &Url) -> Option<String> {
        let host = url.host_str()?;
        let port = url.port().map(|p| format!(":{}", p)).unwrap_or_default();
        Some(format!("{}://{}{}", url.scheme(), host, port))
    }

    fn debugger_address_from_capabilities(capabilities: &Value) -> Option<String> {
        capabilities
            .get("goog:chromeOptions")
            .and_then(|value| value.get("debuggerAddress"))
            .and_then(|value| value.as_str())
            .map(str::to_string)
            .or_else(|| {
                capabilities
                    .get("ms:edgeOptions")
                    .and_then(|value| value.get("debuggerAddress"))
                    .and_then(|value| value.as_str())
                    .map(str::to_string)
            })
    }

    fn load_state_reached(state: crate::core::WaitUntilState, snapshot: &LoadStateSnapshot) -> bool {
        use crate::core::WaitUntilState;

        match state {
            WaitUntilState::Load => snapshot.load,
            WaitUntilState::DomContentLoaded => snapshot.domcontentloaded || snapshot.load,
            WaitUntilState::NetworkIdle => snapshot.network_idle,
            WaitUntilState::Commit => snapshot.commit || snapshot.domcontentloaded || snapshot.load,
        }
    }

    fn element_from_value(
        value: Value,
        handle: Arc<thirtyfour::session::handle::SessionHandle>,
    ) -> Result<WebElement> {
        let element_id = Self::extract_element_id(&value)
            .ok_or_else(|| Error::ActionFailed("Failed to parse element reference".to_string()))?;

        let element_json = json!({W3C_ELEMENT_KEY: element_id});
        WebElement::from_json(element_json, handle).map_err(Error::from)
    }

    fn extract_element_id(value: &Value) -> Option<String> {
        if let Some(id) = Self::extract_element_id_for_key(value, W3C_ELEMENT_KEY) {
            return Some(id);
        }
        if let Some(id) = Self::extract_element_id_for_key(value, W3C_SHADOW_KEY) {
            return Some(id);
        }
        if let Some(id) = Self::extract_element_id_for_key(value, LEGACY_ELEMENT_KEY) {
            return Some(id);
        }

        match value {
            Value::String(id) => Some(id.clone()),
            _ => None,
        }
    }

    fn extract_element_id_for_key(value: &Value, key: &str) -> Option<String> {
        let map = match value.as_object() {
            Some(map) => map,
            None => return None,
        };

        let nested = map.get(key)?;
        match nested {
            Value::String(id) => Some(id.clone()),
            Value::Object(_) => Self::extract_element_id(nested),
            _ => None,
        }
    }

    async fn find_element_raw(&self, selector: &str) -> Result<WebElement> {
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        let response = match driver
            .handle
            .cmd(Command::FindElement(By::Css(selector).into()))
            .await
        {
            Ok(response) => response,
            Err(error) => {
                if matches!(&*error, WebDriverErrorInner::NoSuchElement(_)) {
                    return Err(Error::element_not_found(selector));
                }
                return Err(Error::from(error));
            }
        };

        let value = response.value_json()?;
        Self::element_from_value(value, driver.handle.clone())
    }

    async fn find_elements_raw(&self, selector: &str) -> Result<Vec<WebElement>> {
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        let response = match driver
            .handle
            .cmd(Command::FindElements(By::Css(selector).into()))
            .await
        {
            Ok(response) => response,
            Err(error) => {
                if matches!(&*error, WebDriverErrorInner::NoSuchElement(_))
                    || matches!(&*error, WebDriverErrorInner::NotFound(_, _))
                {
                    return Ok(Vec::new());
                }
                return Err(Error::from(error));
            }
        };

        let values: Vec<Value> = serde_json::from_value(response.value_json()?)?;
        let mut elements = Vec::with_capacity(values.len());
        for value in values {
            elements.push(Self::element_from_value(value, driver.handle.clone())?);
        }
        Ok(elements)
    }

    async fn cdp_websocket_url_for_current_page(&self) -> Result<Option<String>> {
        let capabilities = match self.session_capabilities().await? {
            Some(capabilities) => capabilities,
            None => return Ok(None),
        };

        let debugger_address = match Self::debugger_address_from_capabilities(&capabilities) {
            Some(address) => address,
            None => return Ok(None),
        };

        let current_url = self.current_url().await?;
        let list_url = format!("http://{}/json/list", debugger_address);
        let client = Client::new();

        let response = match client.get(&list_url).send().await {
            Ok(response) => response,
            Err(error) => {
                tracing::debug!("Failed to query CDP targets: {}", error);
                return Ok(None);
            }
        };

        let targets: Value = match response.json().await {
            Ok(targets) => targets,
            Err(error) => {
                tracing::debug!("Failed to parse CDP targets: {}", error);
                return Ok(None);
            }
        };

        let targets = match targets.as_array() {
            Some(targets) => targets,
            None => return Ok(None),
        };

        let mut fallback: Option<&Value> = None;
        for target in targets {
            let target_type = target.get("type").and_then(|value| value.as_str());
            if target_type != Some("page") {
                continue;
            }

            if fallback.is_none() {
                fallback = Some(target);
            }

            let target_url = target.get("url").and_then(|value| value.as_str());
            if target_url == Some(current_url.as_str()) {
                fallback = Some(target);
                break;
            }
        }

        let target = match fallback {
            Some(target) => target,
            None => return Ok(None),
        };

        let ws_url = target
            .get("webSocketDebuggerUrl")
            .and_then(|value| value.as_str())
            .map(str::to_string);

        Ok(ws_url)
    }

    async fn wait_for_load_state_via_cdp(
        &self,
        state: crate::core::WaitUntilState,
        timeout: Duration,
    ) -> Result<Option<()>> {
        let ws_url = match self.cdp_websocket_url_for_current_page().await? {
            Some(url) => url,
            None => return Ok(None),
        };

        let (mut ws_stream, _) = match connect_async(&ws_url).await {
            Ok(result) => result,
            Err(error) => {
                tracing::debug!("Failed to connect to CDP websocket: {}", error);
                return Ok(None);
            }
        };

        let mut next_id = 1u64;

        for (method, params) in [
            ("Page.enable", serde_json::json!({})),
            ("Network.enable", serde_json::json!({})),
            (
                "Page.setLifecycleEventsEnabled",
                serde_json::json!({"enabled": true}),
            ),
        ] {
            let id = next_id;
            next_id += 1;
            let message = serde_json::json!({
                "id": id,
                "method": method,
                "params": params,
            });
            let text = serde_json::to_string(&message)
                .map_err(|e| Error::Serialization(e))?;
            ws_stream
                .send(Message::Text(text.into()))
                .await
                .map_err(|e| Error::ActionFailed(format!("Failed to send CDP command: {}", e)))?;
        }

        let mut snapshot = LoadStateSnapshot::default();
        let mut inflight: HashSet<String> = HashSet::new();
        let mut idle_timer: Option<Pin<Box<Sleep>>> = None;

        let deadline = Instant::now() + timeout;

        loop {
            if Self::load_state_reached(state, &snapshot) {
                return Ok(Some(()));
            }

            let sleep_until = tokio::time::sleep_until(deadline);
            tokio::pin!(sleep_until);

            if let Some(mut idle_sleep) = idle_timer.take() {
                tokio::select! {
                    _ = &mut sleep_until => {
                        return Err(Error::timeout_duration("wait for load state via CDP", timeout));
                    }
                    _ = &mut idle_sleep => {
                        snapshot.network_idle = true;
                        idle_timer = None;
                    }
                    message = ws_stream.next() => {
                        idle_timer = Some(idle_sleep);
                        let message = match message {
                            Some(Ok(message)) => message,
                            Some(Err(error)) => {
                                tracing::debug!("CDP websocket error: {}", error);
                                return Ok(None);
                            }
                            None => return Ok(None),
                        };

                        let text = match message {
                            Message::Text(text) => text.to_string(),
                            Message::Binary(bytes) => {
                                String::from_utf8(bytes.to_vec()).unwrap_or_default()
                            }
                            Message::Ping(_) | Message::Pong(_) | Message::Close(_) | Message::Frame(_) => continue,
                        };

                        let value: Value = match serde_json::from_str(&text) {
                            Ok(value) => value,
                            Err(_) => continue,
                        };

                        let method = value.get("method").and_then(|value| value.as_str());
                        let params = value.get("params");

                        match method {
                            Some("Page.domContentEventFired") => {
                                snapshot.domcontentloaded = true;
                            }
                            Some("Page.loadEventFired") => {
                                snapshot.load = true;
                            }
                            Some("Page.lifecycleEvent") => {
                                if let Some(name) = params.and_then(|p| p.get("name")).and_then(|v| v.as_str()) {
                                    match name {
                                        "DOMContentLoaded" => snapshot.domcontentloaded = true,
                                        "load" => snapshot.load = true,
                                        "networkIdle" | "networkAlmostIdle" => snapshot.network_idle = true,
                                        "commit" => snapshot.commit = true,
                                        _ => {}
                                    }
                                }
                            }
                            Some("Page.frameNavigated") | Some("Page.frameStartedLoading") => {
                                snapshot.commit = true;
                            }
                            Some("Network.requestWillBeSent") => {
                                if let Some(request_id) = params
                                    .and_then(|p| p.get("requestId"))
                                    .and_then(|v| v.as_str())
                                {
                                    inflight.insert(request_id.to_string());
                                    snapshot.network_idle = false;
                                    idle_timer = None;
                                }
                            }
                            Some("Network.loadingFinished") | Some("Network.loadingFailed") => {
                                if let Some(request_id) = params
                                    .and_then(|p| p.get("requestId"))
                                    .and_then(|v| v.as_str())
                                {
                                    inflight.remove(request_id);
                                }

                                if inflight.is_empty() && idle_timer.is_none() {
                                    idle_timer = Some(Box::pin(tokio::time::sleep(Duration::from_millis(500))));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            } else {
                tokio::select! {
                    _ = &mut sleep_until => {
                        return Err(Error::timeout_duration("wait for load state via CDP", timeout));
                    }
                    message = ws_stream.next() => {
                        let message = match message {
                            Some(Ok(message)) => message,
                            Some(Err(error)) => {
                                tracing::debug!("CDP websocket error: {}", error);
                                return Ok(None);
                            }
                            None => return Ok(None),
                        };

                        let text = match message {
                            Message::Text(text) => text.to_string(),
                            Message::Binary(bytes) => {
                                String::from_utf8(bytes.to_vec()).unwrap_or_default()
                            }
                            Message::Ping(_) | Message::Pong(_) | Message::Close(_) | Message::Frame(_) => continue,
                        };

                        let value: Value = match serde_json::from_str(&text) {
                            Ok(value) => value,
                            Err(_) => continue,
                        };

                        let method = value.get("method").and_then(|value| value.as_str());
                        let params = value.get("params");

                        match method {
                            Some("Page.domContentEventFired") => {
                                snapshot.domcontentloaded = true;
                            }
                            Some("Page.loadEventFired") => {
                                snapshot.load = true;
                            }
                            Some("Page.lifecycleEvent") => {
                                if let Some(name) = params.and_then(|p| p.get("name")).and_then(|v| v.as_str()) {
                                    match name {
                                        "DOMContentLoaded" => snapshot.domcontentloaded = true,
                                        "load" => snapshot.load = true,
                                        "networkIdle" | "networkAlmostIdle" => snapshot.network_idle = true,
                                        "commit" => snapshot.commit = true,
                                        _ => {}
                                    }
                                }
                            }
                            Some("Page.frameNavigated") | Some("Page.frameStartedLoading") => {
                                snapshot.commit = true;
                            }
                            Some("Network.requestWillBeSent") => {
                                if let Some(request_id) = params
                                    .and_then(|p| p.get("requestId"))
                                    .and_then(|v| v.as_str())
                                {
                                    inflight.insert(request_id.to_string());
                                    snapshot.network_idle = false;
                                    idle_timer = None;
                                }
                            }
                            Some("Network.loadingFinished") | Some("Network.loadingFailed") => {
                                if let Some(request_id) = params
                                    .and_then(|p| p.get("requestId"))
                                    .and_then(|v| v.as_str())
                                {
                                    inflight.remove(request_id);
                                }

                                if inflight.is_empty() && idle_timer.is_none() {
                                    idle_timer = Some(Box::pin(tokio::time::sleep(Duration::from_millis(500))));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
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

    /// Wait for the page to reach a specific load state
    ///
    /// # Arguments
    /// * `state` - The load state to wait for (Load, DomContentLoaded, NetworkIdle)
    /// * `timeout` - Maximum time to wait
    ///
    ///
    /// # Returns
    /// Ok(()) if the state is reached, Err if timeout or other error occurs
    pub async fn wait_for_load_state(&self, state: crate::core::WaitUntilState, timeout: Duration) -> Result<()> {
        use crate::core::WaitUntilState;
        
        tracing::debug!("Waiting for load state: {:?}", state);
        let start = std::time::Instant::now();
        
        match state {
            WaitUntilState::Load => {
                let guard = self.driver().await?;
                let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;

                match tokio::time::timeout(timeout, driver.title()).await {
                    Ok(Ok(_)) => {
                        tracing::debug!("Load state 'load' reached via WebDriver");
                        Ok(())
                    }
                    Ok(Err(error)) => Err(Error::ActionFailed(format!(
                        "Failed to wait for load state 'load': {}",
                        error
                    ))),
                    Err(_) => Err(Error::timeout_duration(
                        "wait for load state: load",
                        timeout,
                    )),
                }
            }
            WaitUntilState::DomContentLoaded => {
                match self.wait_for_load_state_via_cdp(state, timeout).await {
                    Ok(Some(())) => return Ok(()),
                    Ok(None) => {}
                    Err(Error::BrowserClosed) => return Err(Error::BrowserClosed),
                    Err(error) => {
                        tracing::debug!("CDP load state wait failed, falling back to JS: {}", error);
                    }
                }

                // WebDriver does not expose DOMContentLoaded, fallback to JS polling.
                loop {
                    if start.elapsed() >= timeout {
                        return Err(Error::timeout_duration(
                            "wait for load state: domcontentloaded",
                            timeout,
                        ));
                    }

                    let ready_state = self.execute_script("return document.readyState").await?;
                    let state_str = ready_state.as_str();
                    if state_str == Some("interactive") || state_str == Some("complete") {
                        tracing::debug!("Load state 'domcontentloaded' reached");
                        return Ok(());
                    }

                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            WaitUntilState::NetworkIdle => {
                match self.wait_for_load_state_via_cdp(state, timeout).await {
                    Ok(Some(())) => return Ok(()),
                    Ok(None) => {}
                    Err(Error::BrowserClosed) => return Err(Error::BrowserClosed),
                    Err(error) => {
                        tracing::debug!("CDP load state wait failed, falling back to JS: {}", error);
                    }
                }

                // WebDriver does not expose network idle, fallback to JS polling.
                loop {
                    if start.elapsed() >= timeout {
                        return Err(Error::timeout_duration(
                            "wait for load state: networkidle",
                            timeout,
                        ));
                    }

                    let ready_state = self.execute_script("return document.readyState").await?;
                    if ready_state.as_str() == Some("complete") {
                        tokio::time::sleep(Duration::from_millis(500)).await;

                        let ready_state = self.execute_script("return document.readyState").await?;
                        if ready_state.as_str() == Some("complete") {
                            tracing::debug!("Load state 'networkidle' reached");
                            return Ok(());
                        }
                    }

                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
            WaitUntilState::Commit => {
                match self.wait_for_load_state_via_cdp(state, timeout).await {
                    Ok(Some(())) => return Ok(()),
                    Ok(None) => {}
                    Err(Error::BrowserClosed) => return Err(Error::BrowserClosed),
                    Err(error) => {
                        tracing::debug!("CDP load state wait failed, falling back to JS: {}", error);
                    }
                }

                // WebDriver does not expose commit state, fallback to JS polling.
                loop {
                    if start.elapsed() >= timeout {
                        return Err(Error::timeout_duration("wait for load state: commit", timeout));
                    }

                    let ready_state = self.execute_script("return document.readyState").await?;
                    if ready_state.as_str() != Some("loading") {
                        tracing::debug!("Load state 'commit' reached");
                        return Ok(());
                    }

                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
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

    /// Get the current page source as HTML
    pub async fn page_source(&self) -> Result<String> {
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        let source = driver.source().await?;
        Ok(source)
    }

    /// Find an element by CSS selector
    pub async fn find_element(&self, selector: &str) -> Result<WebElement> {
        self.apply_slow_mo().await;
        self.find_element_raw(selector).await
    }

    /// Find all elements matching a CSS selector
    pub async fn find_elements(&self, selector: &str) -> Result<Vec<WebElement>> {
        self.find_elements_raw(selector).await
    }

    /// Switch to a frame by CSS selector
    ///
    /// This method automatically waits for the iframe to appear before switching.
    ///
    /// # Arguments
    /// * `frame_selector` - CSS selector to locate the iframe element
    pub async fn switch_to_frame_by_selector(&self, frame_selector: &str) -> Result<()> {
        self.apply_slow_mo().await;

        // Wait for the iframe to appear (with retry logic)
        let timeout = Duration::from_secs(30);
        let start = std::time::Instant::now();
        let target_frame = loop {
            match self.find_element_raw(frame_selector).await {
                Ok(frame) => break frame,
                Err(Error::ElementNotFound { .. }) if start.elapsed() >= timeout => {
                    return Err(Error::timeout_duration(
                        &format!("iframe not found: {}", frame_selector),
                        timeout,
                    ));
                }
                Err(Error::ElementNotFound { .. }) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                Err(error) => return Err(error),
            }
        };
        
        tracing::debug!("Found iframe with selector: {}", frame_selector);

        target_frame.enter_frame().await?;
        tracing::debug!("Switched to frame: {}", frame_selector);
        Ok(())
    }

    /// Switch to a frame by WebElement reference
    ///
    /// # Arguments
    /// * `frame_element` - The iframe element to switch to (must be obtained in parent context)
    pub async fn switch_to_frame(&self, frame_element: &WebElement) -> Result<()> {
        self.apply_slow_mo().await;
        let guard = self.driver().await?;
        let _driver = guard.as_ref().ok_or(Error::BrowserClosed)?;

        frame_element.clone().enter_frame().await?;
        tracing::debug!("Switched to frame");
        Ok(())
    }

    /// Switch to the default content (main page context)
    ///
    /// This exits all iframe contexts and returns to the top-level page.
    pub async fn switch_to_default_content(&self) -> Result<()> {
        self.apply_slow_mo().await;
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        driver.enter_default_frame().await?;
        tracing::debug!("Switched to default content");
        Ok(())
    }

    /// Switch to parent frame
    ///
    /// This switches from the current frame to its parent frame.
    pub async fn switch_to_parent_frame(&self) -> Result<()> {
        self.apply_slow_mo().await;
        let guard = self.driver().await?;
        let driver = guard.as_ref().ok_or(Error::BrowserClosed)?;
        driver.enter_parent_frame().await?;
        tracing::debug!("Switched to parent frame");
        Ok(())
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
    ///
    pub async fn browser_version(&self) -> Result<String> {
        let guard = self.driver().await?;
        let _driver = guard.as_ref().ok_or(Error::BrowserClosed)?;

        if let Some(capabilities) = self.session_capabilities().await? {
            if let Some(version) = Self::extract_browser_version(&capabilities) {
                return Ok(version);
            }
        }

        if let Some(requested) = self.requested_capabilities.as_ref() {
            if let Some(version) = Self::extract_browser_version(&serde_json::Value::Object(requested.clone())) {
                return Ok(version);
            }
        }

        let cdp_guard = self.cdp().await?;
        let dev_tools = cdp_guard.as_ref().ok_or(Error::BrowserClosed)?;

        if let Ok(version_info) = dev_tools.execute_cdp("Browser.getVersion").await {
            if let Some(product) = version_info.get("product") {
                if let Some(product_str) = product.as_str() {
                    if let Some(version) = product_str.split('/').nth(1) {
                        return Ok(version.to_string());
                    }
                }
            }
        }

        drop(cdp_guard);
        let result = self.execute_script("return navigator.userAgent").await?;
        if let Some(ua) = result.as_str() {
            if let Some(start) = ua.find("Chrome/") {
                let version_start = start + 7;
                if let Some(end) = ua[version_start..].find(' ') {
                    return Ok(ua[version_start..version_start + end].to_string());
                }
                return Ok(ua[version_start..].to_string());
            }
            if let Some(start) = ua.find("Edg/") {
                let version_start = start + 4;
                if let Some(end) = ua[version_start..].find(' ') {
                    return Ok(ua[version_start..version_start + end].to_string());
                }
                return Ok(ua[version_start..].to_string());
            }

            return Ok(ua.to_string());
        }

        Ok("Unknown".to_string())
    }

    /// Get all cookies via CDP
    ///
    /// Returns all cookies for all origins in the browser context.
    /// This is Chromium-only (uses CDP).
    pub async fn get_cookies(&self) -> Result<Vec<crate::core::storage::CookieState>> {
        use crate::core::storage::{CookieState, SameSite};
        
        let cdp_guard = self.cdp().await?;
        let dev_tools = cdp_guard.as_ref().ok_or(Error::BrowserClosed)?;
        
        let result = dev_tools.execute_cdp("Network.getAllCookies").await
            .map_err(|e| Error::ActionFailed(format!("Failed to get cookies via CDP: {}", e)))?;
        
        // Parse CDP response
        let cookies_json = result.get("cookies")
            .ok_or_else(|| Error::ActionFailed("CDP response missing 'cookies' field".to_string()))?;
        
        let cdp_cookies: Vec<serde_json::Value> = serde_json::from_value(cookies_json.clone())
            .map_err(|e| Error::ActionFailed(format!("Failed to parse cookies: {}", e)))?;
        
        let mut cookies = Vec::new();
        for cookie in cdp_cookies {
            let name = cookie.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let value = cookie.get("value")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let domain = cookie.get("domain")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let path = cookie.get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("/")
                .to_string();
            let expires = cookie.get("expires")
                .and_then(|v| v.as_f64())
                .unwrap_or(-1.0);
            let http_only = cookie.get("httpOnly")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let secure = cookie.get("secure")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            let same_site_str = cookie.get("sameSite")
                .and_then(|v| v.as_str())
                .unwrap_or("Lax");
            let same_site = match same_site_str {
                "Strict" => SameSite::Strict,
                "None" => SameSite::None,
                _ => SameSite::Lax,
            };
            
            cookies.push(CookieState {
                name,
                value,
                domain,
                path,
                expires,
                http_only,
                secure,
                same_site,
            });
        }
        
        Ok(cookies)
    }

    /// Set cookies via CDP
    ///
    /// Sets cookies in the browser context.
    /// This is Chromium-only (uses CDP).
    pub async fn set_cookies(&self, cookies: &[crate::core::storage::CookieState]) -> Result<()> {
        use serde_json::json;
        
        let cdp_guard = self.cdp().await?;
        let dev_tools = cdp_guard.as_ref().ok_or(Error::BrowserClosed)?;
        
        for cookie in cookies {
            let same_site_str = match cookie.same_site {
                crate::core::storage::SameSite::Strict => "Strict",
                crate::core::storage::SameSite::Lax => "Lax",
                crate::core::storage::SameSite::None => "None",
            };
            
            let mut cookie_params = json!({
                "name": cookie.name,
                "value": cookie.value,
                "domain": cookie.domain,
                "path": cookie.path,
                "httpOnly": cookie.http_only,
                "secure": cookie.secure,
                "sameSite": same_site_str,
            });
            
            // Only include expires if it's not a session cookie (-1)
            if cookie.expires >= 0.0 {
                cookie_params["expires"] = json!(cookie.expires);
            }
            
            let params = json!({
                "cookies": [cookie_params]
            });
            
            dev_tools.execute_cdp_with_params("Network.setCookies", params).await
                .map_err(|e| Error::ActionFailed(format!("Failed to set cookie '{}': {}", cookie.name, e)))?;
        }
        
        Ok(())
    }

    async fn get_storage_for_origin_via_cdp(
        &self,
        origin: &str,
    ) -> Result<(Vec<crate::core::storage::NameValue>, Vec<crate::core::storage::NameValue>)> {
        use crate::core::storage::NameValue;
        use serde_json::json;

        let cdp_guard = self.cdp().await?;
        let dev_tools = cdp_guard.as_ref().ok_or(Error::BrowserClosed)?;
        let _ = dev_tools.execute_cdp("DOMStorage.enable").await;

        async fn fetch_items(
            dev_tools: &ChromeDevTools,
            origin: &str,
            is_local_storage: bool,
        ) -> Result<Vec<NameValue>> {
            let params = json!({
                "storageId": {
                    "securityOrigin": origin,
                    "isLocalStorage": is_local_storage
                }
            });

            let result = dev_tools
                .execute_cdp_with_params("DOMStorage.getDOMStorageItems", params)
                .await
                .map_err(|e| {
                    Error::ActionFailed(format!(
                        "Failed to get DOM storage via CDP (local={is_local_storage}): {e}"
                    ))
                })?;

            let items_json = result
                .get("items")
                .ok_or_else(|| Error::ActionFailed("CDP response missing 'items' field".to_string()))?;

            let pairs: Vec<Vec<String>> = serde_json::from_value(items_json.clone())
                .map_err(|e| Error::ActionFailed(format!("Failed to parse DOMStorage items: {e}")))?;

            let mut storage = Vec::with_capacity(pairs.len());
            for pair in pairs {
                if pair.len() == 2 {
                    storage.push(NameValue {
                        name: pair[0].clone(),
                        value: pair[1].clone(),
                    });
                }
            }

            Ok(storage)
        }

        let local_storage = fetch_items(dev_tools, origin, true).await?;
        let session_storage = fetch_items(dev_tools, origin, false).await?;
        Ok((local_storage, session_storage))
    }

    async fn set_storage_for_origin_via_cdp(
        &self,
        origin: &str,
        local_storage: &[crate::core::storage::NameValue],
        session_storage: &[crate::core::storage::NameValue],
    ) -> Result<()> {
        use serde_json::json;

        let cdp_guard = self.cdp().await?;
        let dev_tools = cdp_guard.as_ref().ok_or(Error::BrowserClosed)?;
        let _ = dev_tools.execute_cdp("DOMStorage.enable").await;

        let local_storage_id = json!({
            "securityOrigin": origin,
            "isLocalStorage": true
        });
        let session_storage_id = json!({
            "securityOrigin": origin,
            "isLocalStorage": false
        });

        for item in local_storage {
            let params = json!({
                "storageId": local_storage_id.clone(),
                "key": &item.name,
                "value": &item.value,
            });

            dev_tools
                .execute_cdp_with_params("DOMStorage.setDOMStorageItem", params)
                .await
                .map_err(|e| {
                    Error::ActionFailed(format!(
                        "Failed to set localStorage item '{}': {}",
                        item.name, e
                    ))
                })?;
        }

        for item in session_storage {
            let params = json!({
                "storageId": session_storage_id.clone(),
                "key": &item.name,
                "value": &item.value,
            });

            dev_tools
                .execute_cdp_with_params("DOMStorage.setDOMStorageItem", params)
                .await
                .map_err(|e| {
                    Error::ActionFailed(format!(
                        "Failed to set sessionStorage item '{}': {}",
                        item.name, e
                    ))
                })?;
        }

        Ok(())
    }

    /// Get localStorage and sessionStorage for a given origin
    ///
    /// Requires an open page at the origin.
    /// Returns (localStorage, sessionStorage) as vectors of name-value pairs.
    ///
    pub async fn get_storage_for_origin(&self, origin: &str) -> Result<(Vec<crate::core::storage::NameValue>, Vec<crate::core::storage::NameValue>)> {
        use crate::core::storage::NameValue;
        
        match self.get_storage_for_origin_via_cdp(origin).await {
            Ok(storage) => return Ok(storage),
            Err(Error::BrowserClosed) => return Err(Error::BrowserClosed),
            Err(error) => {
                tracing::debug!("CDP storage read failed, falling back to JS: {}", error);
            }
        }

        // Script to extract localStorage and sessionStorage
        let script = r#"
            return {
                localStorage: Object.keys(localStorage).map(key => ({
                    name: key,
                    value: localStorage.getItem(key)
                })),
                sessionStorage: Object.keys(sessionStorage).map(key => ({
                    name: key,
                    value: sessionStorage.getItem(key)
                }))
            };
        "#;
        
        let result = self.execute_script(script).await
            .map_err(|e| Error::ActionFailed(format!("Failed to get storage for origin '{}': {}", origin, e)))?;
        
        let local_storage_json = result.get("localStorage")
            .ok_or_else(|| Error::ActionFailed("Missing localStorage in response".to_string()))?;
        let session_storage_json = result.get("sessionStorage")
            .ok_or_else(|| Error::ActionFailed("Missing sessionStorage in response".to_string()))?;
        
        let local_storage: Vec<NameValue> = serde_json::from_value(local_storage_json.clone())
            .map_err(|e| Error::ActionFailed(format!("Failed to parse localStorage: {}", e)))?;
        let session_storage: Vec<NameValue> = serde_json::from_value(session_storage_json.clone())
            .map_err(|e| Error::ActionFailed(format!("Failed to parse sessionStorage: {}", e)))?;
        
        Ok((local_storage, session_storage))
    }

    /// Set localStorage and sessionStorage for the current page
    ///
    /// Must be called on a page that is already loaded at the target origin.
    ///
    pub async fn set_storage(&self, local_storage: &[crate::core::storage::NameValue], session_storage: &[crate::core::storage::NameValue]) -> Result<()> {
        if let Ok(url) = self.current_url().await {
            if let Ok(parsed) = Url::parse(&url) {
                if let Some(origin) = Self::origin_from_url(&parsed) {
                    match self
                        .set_storage_for_origin_via_cdp(&origin, local_storage, session_storage)
                        .await
                    {
                        Ok(()) => return Ok(()),
                        Err(Error::BrowserClosed) => return Err(Error::BrowserClosed),
                        Err(error) => {
                            tracing::debug!(
                                "CDP storage write failed, falling back to JS: {}",
                                error
                            );
                        }
                    }
                }
            }
        }

        // Set localStorage
        for item in local_storage {
            let script = format!(
                "localStorage.setItem({}, {});",
                serde_json::to_string(&item.name).unwrap(),
                serde_json::to_string(&item.value).unwrap()
            );
            self.execute_script(&script).await?;
        }
        
        // Set sessionStorage
        for item in session_storage {
            let script = format!(
                "sessionStorage.setItem({}, {});",
                serde_json::to_string(&item.name).unwrap(),
                serde_json::to_string(&item.value).unwrap()
            );
            self.execute_script(&script).await?;
        }
        
        Ok(())
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
