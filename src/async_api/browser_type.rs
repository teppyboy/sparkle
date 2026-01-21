//! Browser type implementation (Chromium, Firefox, WebKit)
//!
//! This module provides the BrowserType interface for launching browsers.

use crate::async_api::browser::Browser;
use crate::core::{ConnectOptions, ConnectOverCdpOptions, Error, LaunchOptions, Result};
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
        tracing::info!("Launching Chromium browser");
        tracing::debug!("Launch options: headless={:?}, devtools={:?}, timeout={:?}", 
            options.headless, options.devtools, options.timeout);
        
        // Build capabilities
        let mut caps = ChromiumCapabilities::new();

        // Handle devtools option - if devtools is true, force headless to false
        let headless = if options.devtools == Some(true) {
            false
        } else {
            options.headless.unwrap_or(true) // Default to headless
        };
        
        caps = caps.headless(headless);

        // Add devtools argument if requested
        if options.devtools == Some(true) {
            tracing::debug!("DevTools enabled, adding --auto-open-devtools-for-tabs argument");
            caps = caps.arg("--auto-open-devtools-for-tabs");
        }

        // Add custom arguments
        for arg in &options.args {
            caps = caps.arg(arg.clone());
        }
        
        if !options.args.is_empty() {
            tracing::debug!("Added {} custom arguments", options.args.len());
        }

        // Build list of default arguments for stability
        let mut default_args = Vec::new();
        
        // Only add --no-sandbox if chromium_sandbox is not explicitly enabled
        if !options.chromium_sandbox.unwrap_or(false) {
            default_args.push("--no-sandbox".to_string());
        }
        default_args.push("--disable-dev-shm-usage".to_string());
        default_args.push("--disable-blink-features=AutomationControlled".to_string());

        // Filter default args based on ignore options
        if options.ignore_all_default_args != Some(true) {
            // Only add default args that are not in the ignore list
            for arg in default_args {
                if !options.ignore_default_args.contains(&arg) {
                    caps = caps.arg(arg);
                }
            }
        }
        // If ignore_all_default_args is true, skip adding all default args

        // Set Chrome binary path
        // Priority: executable_path > channel > find_installed_chrome
        if let Some(executable_path) = options.executable_path {
            // Use provided executable path
            tracing::info!("Using provided executable path: {}", executable_path.display());
            caps = caps.binary(executable_path);
        } else if let Some(channel) = &options.channel {
            // Try to find browser by channel
            tracing::info!("Searching for browser by channel: {}", channel);
            let channel_path = Self::find_chrome_by_channel(channel)?;
            tracing::info!("Found {} at: {}", channel, channel_path.display());
            caps = caps.binary(channel_path);
        } else {
            // Find and set Chrome binary path from installed location
            // This will use the latest versioned Chrome installation
            if let Ok(chrome_path) = ChromeDriverProcess::find_installed_chrome() {
                tracing::info!("Using installed Chrome: {}", chrome_path.display());
                caps = caps.binary(chrome_path);
            } else {
                tracing::warn!("Could not find installed Chrome, will use system default");
            }
        }

        // Add environment variables
        if !options.env.is_empty() {
            tracing::debug!("Setting {} environment variables", options.env.len());
            caps = caps.envs(options.env.clone());
        }

        // Set downloads path if specified
        if let Some(downloads_path) = options.downloads_path {
            tracing::debug!("Setting downloads path: {}", downloads_path.display());
            caps = caps.downloads_path(downloads_path);
        }

        // Set proxy if specified
        if let Some(proxy) = options.proxy {
            tracing::debug!("Configuring proxy: {}", proxy.server);
            caps = caps.proxy(&proxy.server, proxy.bypass.as_deref());
        }

        let capabilities = caps.build();

        // Calculate timeout (default 30 seconds)
        let total_timeout = options.timeout.unwrap_or(std::time::Duration::from_secs(30));
        // Split timeout: 60% for ChromeDriver launch, 40% for browser connection
        let driver_timeout = total_timeout.mul_f32(0.6);
        
        tracing::debug!("Total timeout: {:?}, ChromeDriver timeout: {:?}", total_timeout, driver_timeout);

        // Determine ChromeDriver URL or launch ChromeDriver automatically
        let (chromedriver_url, driver_process) = if let Ok(url) = std::env::var("CHROMEDRIVER_URL") {
            // Use custom ChromeDriver URL from environment variable
            tracing::info!("Using ChromeDriver URL from environment: {}", url);
            (url, None)
        } else {
            // Check if custom ChromeDriver path is provided via CHROMEDRIVER_PATH
            let driver_path = std::env::var("CHROMEDRIVER_PATH")
                .ok()
                .map(PathBuf::from);
            
            if let Some(ref path) = driver_path {
                tracing::info!("Using custom ChromeDriver path: {}", path.display());
            } else {
                tracing::debug!("Launching ChromeDriver from installed location");
            }
            
            // Launch ChromeDriver automatically from installed location or custom path
            let process = ChromeDriverProcess::launch(driver_path, 9515, &options.env, driver_timeout)
                .await
                .map_err(|e| Error::internal(format!("Failed to launch ChromeDriver: {}", e)))?;
            let url = process.url().to_string();
            tracing::info!("ChromeDriver launched successfully at {}", url);
            (url, Some(process))
        };

        // Create WebDriver adapter with slow_mo
        tracing::debug!("Creating WebDriver adapter, slow_mo: {:?}", options.slow_mo);
        let adapter = WebDriverAdapter::create(&chromedriver_url, capabilities, options.slow_mo).await?;

        // Create and return browser with driver process
        tracing::info!("Browser launched successfully");
        Ok(Browser::new(adapter, driver_process))
    }

    /// Connect to an existing browser instance via remote WebDriver
    ///
    /// This method connects to a running WebDriver server (e.g., Selenium Grid,
    /// standalone ChromeDriver, or any WebDriver-compatible endpoint).
    ///
    /// # Arguments
    /// * `endpoint_url` - WebDriver server URL (e.g., "http://localhost:4444")
    /// * `options` - Connection configuration options
    ///
    /// # Returns
    /// A Browser instance connected to the remote browser
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Playwright;
    /// # use sparkle::core::ConnectOptionsBuilder;
    /// # async fn example() -> sparkle::core::Result<()> {
    /// let playwright = Playwright::new().await?;
    /// let options = ConnectOptionsBuilder::default()
    ///     .timeout(std::time::Duration::from_secs(10))
    ///     .build()
    ///     .unwrap();
    /// let browser = playwright.chromium()
    ///     .connect("http://localhost:9515", options)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect(&self, endpoint_url: &str, options: ConnectOptions) -> Result<Browser> {
        match self.name {
            BrowserName::Chromium => self.connect_chromium(endpoint_url, options).await,
            BrowserName::Firefox => Err(Error::not_implemented("Firefox connect support")),
            BrowserName::WebKit => Err(Error::not_implemented("WebKit connect support")),
        }
    }

    /// Connect to Chromium via remote WebDriver
    async fn connect_chromium(&self, endpoint_url: &str, options: ConnectOptions) -> Result<Browser> {
        tracing::info!("Connecting to remote WebDriver at: {}", endpoint_url);
        tracing::debug!("Connect options: timeout={:?}, slow_mo={:?}", options.timeout, options.slow_mo);
        
        // Build capabilities from options
        let mut caps = ChromiumCapabilities::new();

        // Add custom arguments
        for arg in &options.args {
            caps = caps.arg(arg.clone());
        }

        // Set executable path if provided
        if let Some(path) = options.executable_path {
            caps = caps.binary(path);
        }

        // Note: Channel configuration is not yet supported via ChromiumCapabilities
        // This would require extending the capabilities builder
        if options.channel.is_some() {
            tracing::warn!("Channel option is not yet supported for remote connections");
        }

        let capabilities = caps.build();

        // Determine timeout for connection
        let timeout = options.timeout.unwrap_or(std::time::Duration::from_secs(30));
        let start = std::time::Instant::now();
        
        tracing::debug!("Connection timeout: {:?}", timeout);

        // Attempt to connect to the remote WebDriver server
        let adapter = loop {
            match WebDriverAdapter::create(endpoint_url, capabilities.clone(), options.slow_mo).await {
                Ok(adapter) => {
                    tracing::info!("Successfully connected to remote WebDriver");
                    break adapter;
                }
                Err(e) => {
                    if start.elapsed() >= timeout {
                        tracing::error!("Failed to connect after {:?}: {}", timeout, e);
                        return Err(Error::connection_failed(format!(
                            "Failed to connect to WebDriver at '{}' after {:?}: {}",
                            endpoint_url, timeout, e
                        )));
                    }
                    tracing::trace!("Connection attempt failed, retrying: {}", e);
                    // Wait a bit before retrying
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        };

        // Create and return browser without driver process (remote connection)
        Ok(Browser::new(adapter, None))
    }

    /// Connect to a browser via Chrome DevTools Protocol
    ///
    /// This method connects to a Chromium-based browser with `--remote-debugging-port`
    /// enabled. The endpoint should be a WebDriver-compatible URL on the debugging port.
    ///
    /// Note: This implementation uses WebDriver protocol to connect to the CDP endpoint,
    /// then enables CDP features via thirtyfour's CDP extensions. The endpoint must
    /// support both WebDriver and CDP protocols (standard for Chrome with
    /// --remote-debugging-port).
    ///
    /// # Arguments
    /// * `endpoint_url` - CDP endpoint URL (e.g., "http://localhost:9222")
    /// * `options` - Connection configuration options
    ///
    /// # Returns
    /// A Browser instance connected via CDP
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Playwright;
    /// # use sparkle::core::ConnectOverCdpOptionsBuilder;
    /// # async fn example() -> sparkle::core::Result<()> {
    /// // First, launch Chrome with: chrome --remote-debugging-port=9222
    /// let playwright = Playwright::new().await?;
    /// let options = ConnectOverCdpOptionsBuilder::default()
    ///     .timeout(std::time::Duration::from_secs(10))
    ///     .build()
    ///     .unwrap();
    /// let browser = playwright.chromium()
    ///     .connect_over_cdp("http://localhost:9222", options)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_over_cdp(
        &self,
        endpoint_url: &str,
        options: ConnectOverCdpOptions,
    ) -> Result<Browser> {
        match self.name {
            BrowserName::Chromium => self.connect_chromium_cdp(endpoint_url, options).await,
            BrowserName::Firefox => Err(Error::not_implemented("Firefox CDP support")),
            BrowserName::WebKit => Err(Error::not_implemented("WebKit CDP support")),
        }
    }

    /// Connect to Chromium via CDP endpoint
    async fn connect_chromium_cdp(
        &self,
        endpoint_url: &str,
        options: ConnectOverCdpOptions,
    ) -> Result<Browser> {
        tracing::info!("Connecting to CDP endpoint at: {}", endpoint_url);
        tracing::debug!("CDP options: timeout={:?}, slow_mo={:?}", options.timeout, options.slow_mo);
        
        // For CDP connections, we use minimal capabilities
        // The browser is already running with its own configuration
        let caps = ChromiumCapabilities::new().build();

        // Determine timeout for connection
        let timeout = options.timeout.unwrap_or(std::time::Duration::from_secs(30));
        let start = std::time::Instant::now();

        // Attempt to connect to the CDP endpoint via WebDriver
        // Chrome with --remote-debugging-port exposes both CDP and WebDriver protocols
        let adapter = loop {
            match WebDriverAdapter::create(endpoint_url, caps.clone(), options.slow_mo).await {
                Ok(adapter) => break adapter,
                Err(e) => {
                    if start.elapsed() >= timeout {
                        return Err(Error::connection_failed(format!(
                            "Failed to connect to CDP endpoint at '{}' after {:?}: {}. \
                             Make sure Chrome is running with --remote-debugging-port=<port>",
                            endpoint_url, timeout, e
                        )));
                    }
                    // Wait a bit before retrying
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        };

        // Create and return browser without driver process (remote connection)
        // CDP features can be accessed via thirtyfour's ChromeDevTools extension
        Ok(Browser::new(adapter, None))
    }

    /// Get the path to the browser executable
    ///
    /// Returns the path to the installed browser executable.
    /// For Chromium, this searches the ms-playwright directory for installed versions.
    /// For Firefox and WebKit, returns an error as they are not yet supported.
    ///
    /// # Returns
    /// Path to the browser executable
    ///
    /// # Errors
    /// - If no browser installation is found
    /// - If the browser type is not supported (Firefox/WebKit)
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::async_api::Playwright;
    /// # async fn example() -> sparkle::core::Result<()> {
    /// let playwright = Playwright::new().await?;
    /// let path = playwright.chromium().executable_path()?;
    /// println!("Chrome is installed at: {}", path);
    /// # Ok(())
    /// # }
    /// ```
    pub fn executable_path(&self) -> Result<String> {
        match self.name {
            BrowserName::Chromium => {
                let path = Self::find_chrome_executable()?;
                Ok(path.to_string_lossy().to_string())
            }
            BrowserName::Firefox => {
                Err(Error::not_implemented("Firefox support"))
            }
            BrowserName::WebKit => {
                Err(Error::not_implemented("WebKit support"))
            }
        }
    }

    /// Find the installed Chrome executable path
    fn find_chrome_executable() -> Result<PathBuf> {
        // First check CHROME_PATH environment variable
        if let Ok(path) = std::env::var("CHROME_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Get the install directory
        let install_dir = Self::get_install_dir()?;

        // Find chromium-{revision} directories and get the latest version
        let mut versions = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(&install_dir) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with("chromium-") {
                        if let Some(revision) = file_name.strip_prefix("chromium-") {
                            versions.push((revision.to_string(), entry.path()));
                        }
                    }
                }
            }
        }

        if versions.is_empty() {
            return Err(Error::BrowserNotFound(format!(
                "No Chromium installations found in: {}\nRun 'sparkle install chrome' to download Chrome",
                install_dir.display()
            )));
        }

        // Sort versions to get the latest
        versions.sort_by(|a, b| b.0.cmp(&a.0));
        let latest_chrome_dir = &versions[0].1;

        // Find the chrome executable
        let executable_name = if cfg!(windows) {
            "chrome.exe"
        } else if cfg!(target_os = "macos") {
            "Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
        } else {
            "chrome"
        };

        // Look for the executable in common locations
        let possible_paths = vec![
            latest_chrome_dir.join(executable_name),
            latest_chrome_dir.join("chrome-win64").join(executable_name),
            latest_chrome_dir.join("chrome-linux64").join(executable_name),
            latest_chrome_dir.join("chrome-mac-x64").join(executable_name),
            latest_chrome_dir.join("chrome-mac-arm64").join(executable_name),
        ];

        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(Error::BrowserNotFound(format!(
            "Chrome executable not found in: {}",
            latest_chrome_dir.display()
        )))
    }

    /// Find Chrome binary by channel name
    ///
    /// Searches for Chrome variants in standard installation locations and PATH.
    /// Supported channels: chrome, chrome-beta, chrome-dev, chrome-canary, msedge, msedge-beta, msedge-dev
    fn find_chrome_by_channel(channel: &str) -> Result<PathBuf> {
        let executable_names: Vec<String> = if cfg!(windows) {
            match channel {
                "chrome" => vec!["chrome.exe".to_string()],
                "chrome-beta" => vec!["chrome.exe".to_string()],
                "chrome-dev" => vec!["chrome.exe".to_string()],
                "chrome-canary" => vec!["chrome.exe".to_string()],
                "msedge" => vec!["msedge.exe".to_string()],
                "msedge-beta" => vec!["msedge.exe".to_string()],
                "msedge-dev" => vec!["msedge.exe".to_string()],
                _ => return Err(Error::BrowserNotFound(format!("Unknown channel: {}", channel))),
            }
        } else if cfg!(target_os = "macos") {
            match channel {
                "chrome" => vec![
                    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".to_string(),
                ],
                "chrome-beta" => vec![
                    "/Applications/Google Chrome Beta.app/Contents/MacOS/Google Chrome Beta".to_string(),
                ],
                "chrome-dev" => vec![
                    "/Applications/Google Chrome Dev.app/Contents/MacOS/Google Chrome Dev".to_string(),
                ],
                "chrome-canary" => vec![
                    "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary".to_string(),
                ],
                "msedge" => vec![
                    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge".to_string(),
                ],
                "msedge-beta" => vec![
                    "/Applications/Microsoft Edge Beta.app/Contents/MacOS/Microsoft Edge Beta".to_string(),
                ],
                "msedge-dev" => vec![
                    "/Applications/Microsoft Edge Dev.app/Contents/MacOS/Microsoft Edge Dev".to_string(),
                ],
                _ => return Err(Error::BrowserNotFound(format!("Unknown channel: {}", channel))),
            }
        } else {
            // Linux
            match channel {
                "chrome" => vec!["google-chrome".to_string(), "chrome".to_string()],
                "chrome-beta" => vec!["google-chrome-beta".to_string()],
                "chrome-dev" => vec!["google-chrome-unstable".to_string()],
                "msedge" => vec!["microsoft-edge".to_string()],
                "msedge-beta" => vec!["microsoft-edge-beta".to_string()],
                "msedge-dev" => vec!["microsoft-edge-dev".to_string()],
                _ => return Err(Error::BrowserNotFound(format!("Unknown channel: {}", channel))),
            }
        };

        // On Windows, search standard installation directories
        if cfg!(windows) {
            let base_paths = vec![
                std::env::var("ProgramFiles").ok().map(PathBuf::from),
                std::env::var("ProgramFiles(x86)").ok().map(PathBuf::from),
                std::env::var("LOCALAPPDATA").ok().map(PathBuf::from),
            ];

            let app_dirs: Vec<&str> = match channel {
                "chrome" => vec!["Google\\Chrome\\Application"],
                "chrome-beta" => vec!["Google\\Chrome Beta\\Application"],
                "chrome-dev" => vec!["Google\\Chrome Dev\\Application"],
                "chrome-canary" => vec!["Google\\Chrome SxS\\Application"],
                "msedge" => vec!["Microsoft\\Edge\\Application"],
                "msedge-beta" => vec!["Microsoft\\Edge Beta\\Application"],
                "msedge-dev" => vec!["Microsoft\\Edge Dev\\Application"],
                _ => vec![],
            };

            for base in base_paths.iter().flatten() {
                for app_dir in &app_dirs {
                    for exe in &executable_names {
                        let path = base.join(app_dir).join(exe);
                        if path.exists() {
                            return Ok(path);
                        }
                    }
                }
            }
        }

        // On macOS, check absolute paths
        if cfg!(target_os = "macos") {
            for path_str in &executable_names {
                let path = PathBuf::from(path_str);
                if path.exists() {
                    return Ok(path);
                }
            }
        }

        // On Linux/macOS, also search PATH
        if !cfg!(windows) {
            for exe_name in &executable_names {
                if let Ok(output) = std::process::Command::new("which")
                    .arg(exe_name)
                    .output()
                {
                    if output.status.success() {
                        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        let path = PathBuf::from(path_str);
                        if path.exists() {
                            return Ok(path);
                        }
                    }
                }
            }
        }

        Err(Error::BrowserNotFound(format!(
            "Could not find {} browser. Please install {} or provide explicit path via executable_path",
            channel, channel
        )))
    }

    /// Get the Playwright cache directory
    fn get_install_dir() -> Result<PathBuf> {
        // Use Playwright's cache directory structure for compatibility
        // This allows reusing browsers downloaded by Playwright
        
        // Get the platform-specific cache directory
        let cache_base = if cfg!(target_os = "windows") {
            // Windows: %LOCALAPPDATA%
            std::env::var("LOCALAPPDATA")
                .or_else(|_| std::env::var("APPDATA"))
                .map(PathBuf::from)
                .map_err(|_| Error::ActionFailed("Failed to get LOCALAPPDATA or APPDATA".to_string()))?
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Caches
            let home = std::env::var("HOME")
                .map_err(|_| Error::ActionFailed("Failed to get HOME".to_string()))?;
            PathBuf::from(home).join("Library").join("Caches")
        } else {
            // Linux/Unix: ~/.cache
            let home = std::env::var("HOME")
                .map_err(|_| Error::ActionFailed("Failed to get HOME".to_string()))?;
            PathBuf::from(home).join(".cache")
        };
        
        // Append ms-playwright to match Playwright's structure
        Ok(cache_base.join("ms-playwright"))
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

    #[test]
    fn test_executable_path_not_implemented() {
        // Firefox and WebKit should return NotImplemented error
        let firefox = BrowserType::new(BrowserName::Firefox);
        assert!(firefox.executable_path().is_err());
        
        let webkit = BrowserType::new(BrowserName::WebKit);
        assert!(webkit.executable_path().is_err());
    }

    #[test]
    fn test_executable_path_chromium() {
        // Test that Chromium path returns either a valid path or BrowserNotFound error
        let chromium = BrowserType::new(BrowserName::Chromium);
        let result = chromium.executable_path();
        
        // Should either succeed (if Chrome is installed) or return BrowserNotFound
        match result {
            Ok(path) => {
                // If successful, the path should not be empty
                assert!(!path.is_empty());
            }
            Err(e) => {
                // Should be either BrowserNotFound or ActionFailed (for env var issues)
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains("not found") || error_msg.contains("Failed to get"),
                    "Unexpected error: {}",
                    error_msg
                );
            }
        }
    }
}
