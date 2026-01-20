//! List command implementation

use anyhow::Result;
use std::path::PathBuf;

pub async fn run() -> Result<()> {
    println!("Installed Browsers");
    println!("==================\n");

    let install_dir = get_install_dir()?;
    
    if !install_dir.exists() {
        println!("No browsers installed yet.");
        return Ok(());
    }

    // Find all chromium-{revision} installations
    let mut chromium_versions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&install_dir) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.starts_with("chromium-") {
                    if let Some(revision) = file_name.strip_prefix("chromium-") {
                        // Check if chromedriver subdirectory exists
                        let has_driver = entry.path().join("chromedriver").exists();
                        chromium_versions.push((revision.to_string(), entry.path(), has_driver));
                    }
                }
            }
        }
    }

    if chromium_versions.is_empty() {
        println!("Chromium (not installed)");
    } else {
        chromium_versions.sort_by(|a, b| b.0.cmp(&a.0));
        println!("Chromium ({} revision(s) installed)", chromium_versions.len());
        for (revision, path, has_driver) in &chromium_versions {
            let driver_status = if *has_driver { " [with ChromeDriver]" } else { "" };
            println!("  - Revision {}{}: {:?}", revision, driver_status, path);
        }
    }

    Ok(())
}

fn get_install_dir() -> Result<PathBuf> {
    // Use Playwright's cache directory structure for compatibility
    // This allows reusing browsers downloaded by Playwright
    
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
