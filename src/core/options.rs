//! Option types and builders for various Sparkle operations
//!
//! This module provides builder patterns for configuring browser launch options,
//! navigation options, and other configurable operations.

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Stealth configuration for undetectable browser automation
///
/// Patchright-inspired stealth mode that makes Chromium-based browsers
/// undetectable by anti-bot systems. This includes removing automation flags,
/// spoofing navigator properties, and hiding automation artifacts.
///
/// **Note**: Stealth is Chromium-only. Firefox and WebKit are not supported.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthOptions {
    /// Enable stealth mode. Defaults to true for Chromium.
    pub enabled: bool,

    /// Enable console API. When false (default), console is disabled to avoid detection.
    /// Setting to true reduces stealth effectiveness.
    pub console_enabled: bool,

    /// Spoof WebGL vendor and renderer. Defaults to true.
    pub webgl_spoof: bool,

    /// Add canvas noise for fingerprint randomization. Defaults to false.
    pub canvas_noise: bool,

    /// Patch permissions.query to avoid detection. Defaults to true.
    pub permissions_patch: bool,

    /// Automatically align headers (User-Agent, Accept-Language, sec-ch-ua*).
    /// Defaults to true.
    pub header_alignment: bool,

    /// Allow user-provided custom headers even when stealth is on.
    /// Setting to true may reduce stealth effectiveness. Defaults to false.
    pub allow_user_headers: bool,

    /// Locale for Accept-Language header (e.g., "en-US", "en-GB").
    /// If None, uses "en-US,en".
    pub locale: Option<String>,

    /// Timezone ID for emulation (e.g., "America/New_York", "Europe/London").
    /// If None, uses system timezone.
    pub timezone_id: Option<String>,

    /// Geolocation coordinates for emulation (latitude, longitude, accuracy).
    /// If None, geolocation is not emulated.
    pub geolocation: Option<(f64, f64, f64)>,
}

impl Default for StealthOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            console_enabled: false,
            webgl_spoof: true,
            canvas_noise: false,
            permissions_patch: true,
            header_alignment: true,
            allow_user_headers: false,
            locale: None,
            timezone_id: None,
            geolocation: None,
        }
    }
}

/// Options for launching a browser
#[derive(Debug, Clone, Builder, Default)]
#[builder(default, setter(into, strip_option))]
pub struct LaunchOptions {
    /// Whether to run browser in headless mode. Defaults to true.
    pub headless: Option<bool>,

    /// Slows down operations by the specified duration. Useful for debugging.
    pub slow_mo: Option<Duration>,

    /// Maximum time to wait for browser to start. Defaults to 30 seconds.
    pub timeout: Option<Duration>,

    /// Additional arguments to pass to the browser instance.
    #[builder(default)]
    pub args: Vec<String>,

    /// Path to a browser executable to run instead of bundled browser.
    pub executable_path: Option<PathBuf>,

    /// Environment variables to set for the browser process.
    #[builder(default)]
    pub env: HashMap<String, String>,

    /// If specified, accepted downloads are downloaded into this directory.
    pub downloads_path: Option<PathBuf>,

    /// Whether to auto-open DevTools panel. Chromium-only.
    pub devtools: Option<bool>,

    /// Browser distribution channel (e.g., "chrome", "chrome-beta").
    pub channel: Option<String>,

    /// Enable Chromium sandboxing. Defaults to false.
    pub chromium_sandbox: Option<bool>,

    /// If specified, traces are saved into this directory.
    pub traces_dir: Option<PathBuf>,

    /// Close the browser process on SIGHUP. Defaults to true.
    pub handle_sighup: Option<bool>,

    /// Close the browser process on SIGINT. Defaults to true.
    pub handle_sigint: Option<bool>,

    /// Close the browser process on SIGTERM. Defaults to true.
    pub handle_sigterm: Option<bool>,

    /// Network proxy settings.
    pub proxy: Option<ProxySettings>,

    /// If true, ignores all default arguments.
    pub ignore_all_default_args: Option<bool>,

    /// List of default arguments to filter out.
    #[builder(default)]
    pub ignore_default_args: Vec<String>,

    /// Stealth mode configuration (Chromium-only).
    /// Defaults to enabled for undetectable automation.
    pub stealth: Option<StealthOptions>,
}

/// Network proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    /// Proxy server URL (e.g., "http://myproxy.com:3128")
    pub server: String,

    /// Optional comma-separated domains to bypass proxy
    pub bypass: Option<String>,

    /// Username for proxy authentication
    pub username: Option<String>,

    /// Password for proxy authentication
    pub password: Option<String>,
}

/// Options for connecting to an existing browser via remote WebDriver
///
/// This is used with `BrowserType::connect()` to connect to a running
/// WebDriver server (e.g., Selenium Grid, standalone ChromeDriver).
#[derive(Debug, Clone, Builder, Default)]
#[builder(default, setter(into, strip_option))]
pub struct ConnectOptions {
    /// Maximum time to wait for the connection. Defaults to 30 seconds.
    pub timeout: Option<Duration>,

    /// Slows down operations by the specified duration. Useful for debugging.
    pub slow_mo: Option<Duration>,

    /// Additional HTTP headers to send with WebDriver requests
    #[builder(default)]
    pub headers: HashMap<String, String>,

    /// Additional arguments to pass to the browser instance
    #[builder(default)]
    pub args: Vec<String>,

    /// Path to a browser executable to use
    pub executable_path: Option<PathBuf>,

    /// Environment variables to set for the browser process
    #[builder(default)]
    pub env: HashMap<String, String>,

    /// Browser distribution channel (e.g., "chrome", "chrome-beta")
    pub channel: Option<String>,
}

/// Options for connecting to a browser via Chrome DevTools Protocol
///
/// This is used with `BrowserType::connect_over_cdp()` to connect to a
/// Chromium-based browser with `--remote-debugging-port` enabled.
///
/// Note: This implementation connects via WebDriver protocol to the CDP
/// endpoint, then enables CDP features. The endpoint must support both
/// WebDriver and CDP protocols (standard for Chrome with --remote-debugging-port).
#[derive(Debug, Clone, Builder, Default)]
#[builder(default, setter(into, strip_option))]
pub struct ConnectOverCdpOptions {
    /// Maximum time to wait for the connection. Defaults to 30 seconds.
    pub timeout: Option<Duration>,

    /// Slows down operations by the specified duration. Useful for debugging.
    pub slow_mo: Option<Duration>,

    /// Additional HTTP headers to send with CDP requests
    #[builder(default)]
    pub headers: HashMap<String, String>,
}

/// Options for creating a new browser context
#[derive(Debug, Clone, Builder, Default)]
#[builder(default, setter(into, strip_option))]
pub struct BrowserContextOptions {
    /// Whether to automatically download all attachments. Defaults to true.
    pub accept_downloads: Option<bool>,

    /// Toggles bypassing page's Content-Security-Policy. Defaults to false.
    pub bypass_csp: Option<bool>,

    /// Emulates prefers-colors-scheme media feature
    pub color_scheme: Option<ColorScheme>,

    /// Specify device scale factor (DPR). Defaults to 1.
    pub device_scale_factor: Option<f64>,

    /// Additional HTTP headers to send with every request
    #[builder(default)]
    pub extra_http_headers: HashMap<String, String>,

    /// Geolocation settings
    pub geolocation: Option<Geolocation>,

    /// Whether viewport supports touch events
    pub has_touch: Option<bool>,

    /// HTTP credentials for authentication
    pub http_credentials: Option<HttpCredentials>,

    /// Whether to ignore HTTPS errors. Defaults to false.
    pub ignore_https_errors: Option<bool>,

    /// Whether the meta viewport tag is taken into account
    pub is_mobile: Option<bool>,

    /// Whether to enable JavaScript. Defaults to true.
    pub java_script_enabled: Option<bool>,

    /// Locale (e.g., "en-GB", "de-DE")
    pub locale: Option<String>,

    /// Whether to emulate network being offline. Defaults to false.
    pub offline: Option<bool>,

    /// Permissions to grant to all pages
    #[builder(default)]
    pub permissions: Vec<String>,

    /// Proxy settings for this context
    pub proxy: Option<ProxySettings>,

    /// Specific user agent to use
    pub user_agent: Option<String>,

    /// Viewport size
    pub viewport: Option<ViewportSize>,

    /// Timezone ID
    pub timezone_id: Option<String>,

    /// Base URL for relative navigation
    pub base_url: Option<String>,

    /// Enable strict selectors mode
    pub strict_selectors: Option<bool>,

    /// Service workers setting
    pub service_workers: Option<ServiceWorkersPolicy>,

    /// Whether to record HAR
    pub record_har_path: Option<PathBuf>,

    /// Whether to record video
    pub record_video_dir: Option<PathBuf>,

    /// Video size
    pub record_video_size: Option<ViewportSize>,

    /// Control focus behavior (Patchright extension). Defaults to true.
    pub focus_control: Option<bool>,

    /// Stealth mode configuration (Chromium-only).
    /// Defaults to enabled for undetectable automation.
    pub stealth: Option<StealthOptions>,

    /// Populate context with given storage state.
    /// This can be a path to a JSON file or an inline StorageState object.
    /// Allows restoring cookies, localStorage, and sessionStorage from a previous session.
    pub storage_state: Option<crate::core::storage::StorageStateSource>,
}

/// Color scheme preference
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorScheme {
    Light,
    Dark,
    NoPreference,
}

/// Geolocation coordinates
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Geolocation {
    /// Latitude between -90 and 90
    pub latitude: f64,
    /// Longitude between -180 and 180
    pub longitude: f64,
    /// Non-negative accuracy value. Defaults to 0.
    pub accuracy: Option<f64>,
}

/// HTTP authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCredentials {
    pub username: String,
    pub password: String,
    /// Restrict to specific origin
    pub origin: Option<String>,
}

/// Viewport size configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ViewportSize {
    /// Page width in pixels
    pub width: u32,
    /// Page height in pixels
    pub height: u32,
}

impl Default for ViewportSize {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
        }
    }
}

/// Service workers policy
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceWorkersPolicy {
    Allow,
    Block,
}

/// Options for page.goto() navigation
#[derive(Debug, Clone, Builder, Default)]
#[builder(default, setter(into, strip_option))]
pub struct NavigationOptions {
    /// Maximum navigation time in milliseconds. Defaults to 30 seconds.
    pub timeout: Option<Duration>,

    /// When to consider navigation succeeded
    pub wait_until: Option<WaitUntilState>,

    /// Referer header value
    pub referer: Option<String>,
}

/// Navigation wait state
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WaitUntilState {
    /// Wait for the load event
    Load,
    /// Wait for DOMContentLoaded event
    DomContentLoaded,
    /// Wait until there are no network connections for at least 500ms
    NetworkIdle,
    /// Wait for the commit event
    Commit,
}

impl Default for WaitUntilState {
    fn default() -> Self {
        Self::Load
    }
}

/// Options for element click actions
#[derive(Debug, Clone, Builder, Default)]
#[builder(default, setter(into, strip_option))]
pub struct ClickOptions {
    /// Maximum time to wait. Defaults to 30 seconds.
    pub timeout: Option<Duration>,

    /// Whether to bypass actionability checks
    pub force: Option<bool>,

    /// Mouse button to use
    pub button: Option<MouseButton>,

    /// Number of times to click. Defaults to 1.
    pub click_count: Option<u32>,

    /// Time to wait between mousedown and mouseup
    pub delay: Option<Duration>,

    /// Modifier keys to press
    #[builder(default)]
    pub modifiers: Vec<KeyboardModifier>,

    /// Click position relative to element
    pub position: Option<Position>,

    /// When true, performs actionability checks without performing action
    pub trial: Option<bool>,

    /// Whether to wait for initiated navigations to complete
    pub no_wait_after: Option<bool>,
}

/// Mouse button types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Keyboard modifier keys
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KeyboardModifier {
    Alt,
    Control,
    Meta,
    Shift,
}

/// Position coordinates
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// Options for typing text
#[derive(Debug, Clone, Builder, Default)]
#[builder(default, setter(into, strip_option))]
pub struct TypeOptions {
    /// Time to wait between key presses
    pub delay: Option<Duration>,

    /// Maximum time to wait. Defaults to 30 seconds.
    pub timeout: Option<Duration>,

    /// Whether to wait for initiated navigations to complete
    pub no_wait_after: Option<bool>,
}

/// Screenshot options
#[derive(Debug, Clone, Builder, Default)]
#[builder(default, setter(into, strip_option))]
pub struct ScreenshotOptions {
    /// File path to save the screenshot
    pub path: Option<PathBuf>,

    /// Screenshot type
    pub r#type: Option<ScreenshotType>,

    /// JPEG quality (0-100), only for JPEG
    pub quality: Option<u8>,

    /// Capture full scrollable page
    pub full_page: Option<bool>,

    /// Hides default white background
    pub omit_background: Option<bool>,

    /// Maximum time to wait
    pub timeout: Option<Duration>,
}

/// Screenshot format type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScreenshotType {
    Png,
    Jpeg,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_options_builder() {
        let opts = LaunchOptionsBuilder::default()
            .headless(true)
            .slow_mo(Duration::from_millis(100))
            .build()
            .unwrap();

        assert_eq!(opts.headless, Some(true));
        assert_eq!(opts.slow_mo, Some(Duration::from_millis(100)));
    }

    #[test]
    fn test_viewport_default() {
        let viewport = ViewportSize::default();
        assert_eq!(viewport.width, 1280);
        assert_eq!(viewport.height, 720);
    }

    #[test]
    fn test_navigation_options() {
        let opts = NavigationOptionsBuilder::default()
            .timeout(Duration::from_secs(10))
            .wait_until(WaitUntilState::NetworkIdle)
            .build()
            .unwrap();

        assert_eq!(opts.timeout, Some(Duration::from_secs(10)));
        assert!(matches!(opts.wait_until, Some(WaitUntilState::NetworkIdle)));
    }
}
