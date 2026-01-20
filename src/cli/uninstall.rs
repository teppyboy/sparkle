//! Uninstall command implementation

use anyhow::Result;
use std::path::PathBuf;

pub async fn run(browser: &str) -> Result<()> {
    println!("Uninstalling {}...\n", browser);

    let install_dir = get_install_dir()?;

    if !install_dir.exists() {
        println!("No browsers installed.");
        return Ok(());
    }

    match browser.to_lowercase().as_str() {
        "chromium" | "chrome" => {
            uninstall_chrome(&install_dir)?;
        }
        "chromedriver" => {
            uninstall_chromedriver(&install_dir)?;
        }
        "all" => {
            uninstall_chrome(&install_dir)?;
            uninstall_chromedriver(&install_dir)?;
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown browser: {}", browser));
        }
    }

    println!("\nUninstall complete!");
    Ok(())
}

fn uninstall_chrome(install_dir: &PathBuf) -> Result<()> {
    let chrome_dir = install_dir.join("chrome");
    if chrome_dir.exists() {
        std::fs::remove_dir_all(&chrome_dir)?;
        println!("✓ Chrome uninstalled");
    } else {
        println!("Chrome is not installed");
    }
    Ok(())
}

fn uninstall_chromedriver(install_dir: &PathBuf) -> Result<()> {
    let driver_dir = install_dir.join("chromedriver");
    if driver_dir.exists() {
        std::fs::remove_dir_all(&driver_dir)?;
        println!("✓ ChromeDriver uninstalled");
    } else {
        println!("ChromeDriver is not installed");
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
