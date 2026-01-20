//! Install command implementation

use super::{Downloader, Platform};
use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

/// Playwright's browsers.json structure
#[derive(Debug, Deserialize)]
struct PlaywrightBrowsersJson {
    browsers: Vec<PlaywrightBrowser>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlaywrightBrowser {
    name: String,
    revision: String,
    browser_version: Option<String>,
    install_by_default: bool,
}

/// Get the latest Playwright Chromium browser info
/// This fetches the default chromium version that Playwright uses
async fn get_latest_playwright_chromium() -> Result<PlaywrightBrowser> {
    // Fetch Playwright's browsers.json from GitHub
    let url = "https://raw.githubusercontent.com/microsoft/playwright/main/packages/playwright-core/browsers.json";
    
    let response = reqwest::get(url).await?;
    let browsers_json: PlaywrightBrowsersJson = response.json().await?;
    
    // Find the default chromium installation (installByDefault = true)
    for browser in browsers_json.browsers {
        if browser.name == "chromium" && browser.install_by_default {
            return Ok(browser);
        }
    }
    
    Err(anyhow::anyhow!("Could not find default Chromium in Playwright's browsers.json"))
}

/// Create Playwright marker file to prevent removal
/// Playwright checks for INSTALLATION_COMPLETE to know not to delete a browser
fn create_marker_file(browser_dir: &std::path::Path) -> Result<()> {
    let marker_path = browser_dir.join("INSTALLATION_COMPLETE");
    std::fs::write(&marker_path, "")?;
    Ok(())
}

/// Create Playwright link file and browsers.json to register this installation
/// This prevents Playwright from removing our browsers during cleanup
fn create_playwright_link(install_dir: &PathBuf, revision: &str, version: &str) -> Result<()> {
    // Create .links directory
    let links_dir = install_dir.join(".links");
    std::fs::create_dir_all(&links_dir)?;
    
    // Create a unique link file for Sparkle installations
    // Using a hash of "sparkle" to create a consistent identifier
    let link_name = "sparkle-installation";
    let link_path = links_dir.join(link_name);
    
    // The link file should contain the path to a directory with browsers.json
    // We'll create a .sparkle directory to hold our browsers.json
    let sparkle_config_dir = install_dir.join(".sparkle");
    std::fs::create_dir_all(&sparkle_config_dir)?;
    
    // Create a browsers.json file that describes our installation
    let browsers_json = serde_json::json!({
        "comment": "Sparkle browser installation - DO NOT EDIT",
        "browsers": [{
            "name": "chromium",
            "revision": revision,
            "browserVersion": version,
            "installByDefault": true
        }]
    });
    
    let browsers_json_path = sparkle_config_dir.join("browsers.json");
    std::fs::write(&browsers_json_path, serde_json::to_string_pretty(&browsers_json)?)?;
    
    // Write the link file pointing to our config directory
    std::fs::write(&link_path, sparkle_config_dir.to_string_lossy().as_bytes())?;
    
    println!("Created Playwright link file for browser protection");
    
    Ok(())
}

pub async fn run(browser: &str, skip_driver: bool, force: bool) -> Result<()> {
    println!("Sparkle Browser Installer");
    println!("=========================\n");

    let platform = Platform::detect()?;
    println!("Detected platform: {}", platform);

    let install_dir = get_install_dir()?;
    println!("Install directory: {:?}\n", install_dir);

    let downloader = Downloader::new();

    // Fetch Playwright's latest Chromium version directly
    println!("Fetching latest Playwright Chromium version...");
    let playwright_browser = get_latest_playwright_chromium().await?;
    let revision = playwright_browser.revision;
    let version = playwright_browser.browser_version
        .ok_or_else(|| anyhow::anyhow!("No browser version found in Playwright's browsers.json"))?;
    
    println!("Latest Playwright Chromium:");
    println!("  Revision: {}", revision);
    println!("  Chrome version: {}\n", version);

    match browser.to_lowercase().as_str() {
        "chromium" | "chrome" => {
            install_chrome(&downloader, &platform, &version, &revision, &install_dir, force).await?;
            // Install ChromeDriver by default unless --skip-driver is specified
            if !skip_driver {
                install_chromedriver(&downloader, &platform, &version, &revision, &install_dir, force).await?;
            }
        }
        "all" => {
            install_chrome(&downloader, &platform, &version, &revision, &install_dir, force).await?;
            install_chromedriver(&downloader, &platform, &version, &revision, &install_dir, force).await?;
        }
        _ => {
            return Err(anyhow::anyhow!("Unknown browser: {}", browser));
        }
    }

    println!("\nInstallation complete!");
    println!("\nInstalled:");
    println!("  Chromium revision {} (Chrome {}): {:?}", revision, version, install_dir.join(format!("chromium-{}", revision)));
    if !skip_driver {
        println!("  ChromeDriver: {:?}", install_dir.join(format!("chromium-{}", revision)).join("chromedriver"));
    }

    Ok(())
}

async fn install_chrome(
    downloader: &Downloader,
    platform: &Platform,
    version: &str,
    revision: &str,
    install_dir: &PathBuf,
    force: bool,
) -> Result<()> {
    // Use Playwright-style naming: chromium-{revision}
    let chrome_dir = install_dir.join(format!("chromium-{}", revision));
    
    if chrome_dir.exists() && !force {
        println!("Chromium {} (Chrome {}) is already installed. Use --force to reinstall.", revision, version);
        return Ok(());
    }

    if chrome_dir.exists() {
        std::fs::remove_dir_all(&chrome_dir)?;
    }

    let url = platform.chrome_download_url(version);
    downloader.install_chrome(version, &url, &chrome_dir).await?;
    
    // Create Playwright marker file to prevent removal by Playwright
    create_marker_file(&chrome_dir)?;
    println!("Created INSTALLATION_COMPLETE marker file");
    
    // Create Playwright link file to register this installation
    create_playwright_link(install_dir, revision, version)?;

    Ok(())
}

async fn install_chromedriver(
    downloader: &Downloader,
    platform: &Platform,
    version: &str,
    revision: &str,
    install_dir: &PathBuf,
    force: bool,
) -> Result<()> {
    // ChromeDriver uses same revision as chromium
    let driver_dir = install_dir.join(format!("chromium-{}", revision)).join("chromedriver");
    
    if driver_dir.exists() && !force {
        println!("ChromeDriver {} (Chrome {}) is already installed. Use --force to reinstall.", revision, version);
        return Ok(());
    }

    // Create parent chromium directory if it doesn't exist
    if let Some(parent) = driver_dir.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
            // Create marker file for the chromium directory
            create_marker_file(parent)?;
        }
    }

    if driver_dir.exists() {
        std::fs::remove_dir_all(&driver_dir)?;
    }

    let url = platform.chromedriver_download_url(version);
    downloader.install_chromedriver(version, &url, &driver_dir).await?;

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
