//! ChromeDriver process management
//!
//! This module handles launching and managing the ChromeDriver process.

use anyhow::{Context, Result};
use std::collections::HashMap;
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
    /// * `env` - Environment variables to set for the ChromeDriver process
    /// * `timeout` - Maximum time to wait for ChromeDriver to start
    pub async fn launch(
        executable_path: Option<PathBuf>, 
        port: u16, 
        env: &HashMap<String, String>,
        timeout: Duration,
    ) -> Result<Self> {
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

        // Launch ChromeDriver process with environment variables
        let mut cmd = Command::new(&driver_path);
        cmd.arg(format!("--port={}", port));
        
        // Set environment variables
        cmd.envs(env);
        
        let process = cmd.spawn()
            .context(format!("Failed to launch ChromeDriver from {:?}", driver_path))?;

        // Wait for ChromeDriver to be ready
        let client = reqwest::Client::new();
        let start = std::time::Instant::now();
        
        loop {
            if let Ok(response) = client.get(&format!("{}/status", url)).send().await {
                if response.status().is_success() {
                    println!("ChromeDriver launched successfully on {}", url);
                    return Ok(Self { process, url });
                }
            }
            
            if start.elapsed() >= timeout {
                return Err(anyhow::anyhow!(
                    "ChromeDriver failed to start within {:?}",
                    timeout
                ));
            }
            
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get the ChromeDriver URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Find the installed Chrome binary path (latest version)
    pub fn find_installed_chrome() -> Result<PathBuf> {
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
            return Err(anyhow::anyhow!(
                "No Chromium installations found in: {:?}\nRun 'sparkle install chrome' to download Chrome",
                install_dir
            ));
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
                println!("Using Chrome: {:?}", path);
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!(
            "Chrome executable not found in: {:?}",
            latest_chrome_dir
        ))
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

        // Find the chromedriver executable in chromium-{revision}/chromedriver directories
        let executable_name = if cfg!(windows) {
            "chromedriver.exe"
        } else {
            "chromedriver"
        };

        // Look for chromium-{revision} directories and find the latest version
        let mut versions = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(&install_dir) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with("chromium-") {
                        // Check if chromedriver subdirectory exists
                        let driver_subdir = entry.path().join("chromedriver");
                        if driver_subdir.exists() {
                            if let Some(revision) = file_name.strip_prefix("chromium-") {
                                versions.push((revision.to_string(), driver_subdir));
                            }
                        }
                    }
                }
            }
        }

        if versions.is_empty() {
            return Err(anyhow::anyhow!(
                "No ChromeDriver installations found in: {:?}\nRun 'sparkle install chrome' to download ChromeDriver",
                install_dir
            ));
        }

        // Sort versions to get the latest (semantic versioning comparison)
        versions.sort_by(|a, b| {
            // Simple version comparison - you could use a proper semver crate for production
            b.0.cmp(&a.0)
        });

        let latest_driver_dir = &versions[0].1;
        
        // Look for the executable in common locations within the driver directory
        let possible_paths = vec![
            latest_driver_dir.join(executable_name),
            latest_driver_dir.join("chromedriver-win64").join(executable_name),
            latest_driver_dir.join("chromedriver-linux64").join(executable_name),
            latest_driver_dir.join("chromedriver-mac-x64").join(executable_name),
            latest_driver_dir.join("chromedriver-mac-arm64").join(executable_name),
        ];

        for path in possible_paths {
            if path.exists() {
                println!("Using ChromeDriver: {:?}", path);
                return Ok(path);
            }
        }

        Err(anyhow::anyhow!(
            "ChromeDriver executable not found in: {:?}\nRun 'sparkle install chrome' to download ChromeDriver",
            latest_driver_dir
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
