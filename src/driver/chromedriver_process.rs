//! ChromeDriver process management
//!
//! This module handles launching and managing the ChromeDriver process.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

/// ChromeDriver process manager
pub struct ChromeDriverProcess {
    process: Child,
    url: String,
}

impl ChromeDriverProcess {
    /// Launch ChromeDriver and return the process manager
    ///
    /// # Arguments
    /// * `executable_path` - Optional path to ChromeDriver executable. If None, uses the installed path.
    /// * `port` - Port to run ChromeDriver on (default: 9515)
    pub async fn launch(executable_path: Option<PathBuf>, port: u16) -> Result<Self> {
        let driver_path = if let Some(path) = executable_path {
            path
        } else {
            Self::find_installed_chromedriver()?
        };

        // Verify the executable exists
        if !driver_path.exists() {
            return Err(anyhow::anyhow!(
                "ChromeDriver executable not found at: {:?}\nRun 'sparkle install chrome' to download ChromeDriver",
                driver_path
            ));
        }

        let url = format!("http://localhost:{}", port);

        // Launch ChromeDriver process
        let process = Command::new(&driver_path)
            .arg(format!("--port={}", port))
            .spawn()
            .context(format!("Failed to launch ChromeDriver from {:?}", driver_path))?;

        // Wait for ChromeDriver to be ready
        let mut retries = 30; // 3 seconds total (30 * 100ms)
        let client = reqwest::Client::new();
        
        while retries > 0 {
            if let Ok(response) = client.get(&format!("{}/status", url)).send().await {
                if response.status().is_success() {
                    println!("ChromeDriver launched successfully on {}", url);
                    return Ok(Self { process, url });
                }
            }
            sleep(Duration::from_millis(100)).await;
            retries -= 1;
        }

        Err(anyhow::anyhow!(
            "ChromeDriver failed to start within 3 seconds"
        ))
    }

    /// Get the ChromeDriver URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Find the installed ChromeDriver executable path
    fn find_installed_chromedriver() -> Result<PathBuf> {
        // First check CHROMEDRIVER_PATH environment variable
        if let Ok(path) = std::env::var("CHROMEDRIVER_PATH") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Get the install directory (same as used by CLI install command)
        let install_dir = Self::get_install_dir()?;
        let driver_dir = install_dir.join("chromedriver");

        // Find the chromedriver executable in the driver directory
        let executable_name = if cfg!(windows) {
            "chromedriver.exe"
        } else {
            "chromedriver"
        };

        // Look for the executable in common locations within the driver directory
        let possible_paths = vec![
            driver_dir.join(executable_name),
            driver_dir.join("chromedriver-win64").join(executable_name),
            driver_dir.join("chromedriver-linux64").join(executable_name),
            driver_dir.join("chromedriver-mac-x64").join(executable_name),
            driver_dir.join("chromedriver-mac-arm64").join(executable_name),
        ];

        for path in possible_paths {
            if path.exists() {
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!(
            "ChromeDriver not found in installation directory: {:?}\nRun 'sparkle install chrome' to download ChromeDriver",
            driver_dir
        ))
    }

    /// Get the install directory (same as CLI install command)
    fn get_install_dir() -> Result<PathBuf> {
        // Use Playwright's cache directory structure for compatibility
        
        // Get the platform-specific cache directory
        let cache_base = if cfg!(target_os = "windows") {
            // Windows: %LOCALAPPDATA%
            std::env::var("LOCALAPPDATA")
                .or_else(|_| std::env::var("APPDATA"))
                .map(PathBuf::from)?
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Caches
            let home = std::env::var("HOME")?;
            PathBuf::from(home).join("Library").join("Caches")
        } else {
            // Linux/Unix: ~/.cache
            let home = std::env::var("HOME")?;
            PathBuf::from(home).join(".cache")
        };
        
        // Append ms-playwright to match Playwright's structure
        Ok(cache_base.join("ms-playwright"))
    }
}

impl Drop for ChromeDriverProcess {
    fn drop(&mut self) {
        // Kill the ChromeDriver process when the manager is dropped
        let _ = self.process.kill();
    }
}
