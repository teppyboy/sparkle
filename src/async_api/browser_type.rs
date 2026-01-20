//! Browser type implementation (Chromium, Firefox, WebKit)
//!
//! This module provides the BrowserType interface for launching browsers.

use crate::async_api::browser::Browser;
use crate::core::{Error, LaunchOptions, Result};
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
        // Build capabilities
        let mut caps = ChromiumCapabilities::new();

        // Set headless mode
        if let Some(headless) = options.headless {
            caps = caps.headless(headless);
        } else {
            caps = caps.headless(true); // Default to headless
        }

        // Add custom arguments
        for arg in &options.args {
            caps = caps.arg(arg.clone());
        }

        // Add common arguments for stability
        caps = caps
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-blink-features=AutomationControlled");

        // Find and set Chrome binary path from installed location
        // This will use the latest versioned Chrome installation
        if let Ok(chrome_path) = ChromeDriverProcess::find_installed_chrome() {
            caps = caps.binary(chrome_path);
        }

        let capabilities = caps.build();

        // Determine ChromeDriver URL or launch ChromeDriver automatically
        let (chromedriver_url, driver_process) = if let Ok(url) = std::env::var("CHROMEDRIVER_URL") {
            // Use custom ChromeDriver URL from environment variable
            (url, None)
        } else {
            // Check if custom ChromeDriver path is provided via CHROMEDRIVER_PATH
            let driver_path = std::env::var("CHROMEDRIVER_PATH")
                .ok()
                .map(PathBuf::from);
            
            // Launch ChromeDriver automatically from installed location or custom path
            let process = ChromeDriverProcess::launch(driver_path, 9515)
                .await
                .map_err(|e| Error::internal(format!("Failed to launch ChromeDriver: {}", e)))?;
            let url = process.url().to_string();
            (url, Some(process))
        };

        // Create WebDriver adapter
        let adapter = WebDriverAdapter::create(&chromedriver_url, capabilities).await?;

        // Create and return browser with driver process
        Ok(Browser::new(adapter, driver_process))
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
