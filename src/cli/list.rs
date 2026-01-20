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

    let chrome_dir = install_dir.join("chrome");
    if chrome_dir.exists() {
        println!("✓ Chrome");
        println!("  Location: {:?}", chrome_dir);
    } else {
        println!("✗ Chrome (not installed)");
    }

    let driver_dir = install_dir.join("chromedriver");
    if driver_dir.exists() {
        println!("\n✓ ChromeDriver");
        println!("  Location: {:?}", driver_dir);
    } else {
        println!("\n✗ ChromeDriver (not installed)");
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
