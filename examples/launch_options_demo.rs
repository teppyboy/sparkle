//! Launch Options Demo
//!
//! This example demonstrates the various launch options available in Sparkle.
//! It shows how to configure:
//! - Headless mode
//! - DevTools auto-open
//! - Custom arguments
//! - Timeout configuration
//! - Slow motion for debugging
//! - Downloads path
//! - Proxy settings
//! - Environment variables
//! - Chromium sandbox
//! - Channel selection
//!
//! # Prerequisites
//! ChromeDriver will be automatically launched from the installed location.
//! 
//! Install ChromeDriver if not already installed:
//! ```bash
//! sparkle install chrome
//! ```
//!
//! # Logging
//! To see detailed logs during execution:
//! ```bash
//! export SPARKLE_LOG_LEVEL=debug  # Options: trace, debug, info, warn, error
//! cargo run --example launch_options_demo
//! ```

use sparkle::prelude::*;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging from SPARKLE_LOG_LEVEL environment variable
    init_logging();

    println!("=== Sparkle Launch Options Demo ===\n");

    // Create Playwright instance
    let playwright = Playwright::new().await?;
    println!("✓ Playwright initialized\n");

    // Example 1: Basic headless launch (default)
    println!("Example 1: Basic headless launch");
    println!("----------------------------------");
    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .build()
        .unwrap();

    let browser = playwright.chromium().launch(options).await?;
    let version = browser.version().await?;
    println!("✓ Browser launched in headless mode");
    println!("  Version: {}", version);
    browser.close().await?;
    println!("✓ Browser closed\n");

    // Example 2: Launch with DevTools open (non-headless)
    println!("Example 2: Launch with DevTools");
    println!("--------------------------------");
    let options = LaunchOptionsBuilder::default()
        .devtools(true)  // This automatically sets headless=false
        .timeout(Duration::from_secs(45))
        .build()
        .unwrap();

    println!("  Note: devtools=true automatically disables headless mode");
    println!("  (Skipping actual launch to avoid opening browser window)\n");

    // Example 3: Launch with slow motion for debugging
    println!("Example 3: Slow motion debugging");
    println!("----------------------------------");
    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .slow_mo(Duration::from_millis(500))  // 500ms delay between operations
        .build()
        .unwrap();

    let browser = playwright.chromium().launch(options).await?;
    let page = browser.new_page().await?;
    
    println!("✓ Browser launched with 500ms slow motion");
    println!("  Navigating to example.com (you'll see the delay)...");
    
    page.goto("https://example.com", Default::default()).await?;
    let title = page.title().await?;
    
    println!("✓ Page title: {}", title);
    browser.close().await?;
    println!("✓ Browser closed\n");

    // Example 4: Custom arguments and sandbox configuration
    println!("Example 4: Custom arguments");
    println!("---------------------------");
    let mut custom_args = Vec::new();
    custom_args.push("--window-size=1920,1080".to_string());
    custom_args.push("--start-maximized".to_string());

    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .args(custom_args)
        .chromium_sandbox(false)  // Disable sandbox (default is false)
        .build()
        .unwrap();

    let browser = playwright.chromium().launch(options).await?;
    println!("✓ Browser launched with custom window size");
    println!("  Arguments: --window-size=1920,1080, --start-maximized");
    println!("  Sandbox: disabled");
    browser.close().await?;
    println!("✓ Browser closed\n");

    // Example 5: Downloads path configuration
    println!("Example 5: Downloads path");
    println!("-------------------------");
    let downloads_dir = std::env::temp_dir().join("sparkle_downloads");
    std::fs::create_dir_all(&downloads_dir)?;

    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .downloads_path(downloads_dir.clone())
        .build()
        .unwrap();

    let browser = playwright.chromium().launch(options).await?;
    println!("✓ Browser launched with downloads path: {}", downloads_dir.display());
    browser.close().await?;
    println!("✓ Browser closed\n");

    // Example 6: Proxy configuration
    println!("Example 6: Proxy settings");
    println!("-------------------------");
    let proxy = ProxySettings {
        server: "http://proxy.example.com:8080".to_string(),
        bypass: Some("localhost,127.0.0.1".to_string()),
        username: None,
        password: None,
    };

    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .proxy(proxy)
        .build()
        .unwrap();

    println!("  (Skipping actual launch - proxy server doesn't exist)");
    println!("  Configuration: http://proxy.example.com:8080");
    println!("  Bypass: localhost,127.0.0.1\n");

    // Example 7: Environment variables
    println!("Example 7: Environment variables");
    println!("---------------------------------");
    let mut env_vars = HashMap::new();
    env_vars.insert("CUSTOM_VAR".to_string(), "test_value".to_string());
    env_vars.insert("DEBUG_MODE".to_string(), "true".to_string());

    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .env(env_vars)
        .build()
        .unwrap();

    let browser = playwright.chromium().launch(options).await?;
    println!("✓ Browser launched with custom environment variables");
    println!("  CUSTOM_VAR=test_value");
    println!("  DEBUG_MODE=true");
    browser.close().await?;
    println!("✓ Browser closed\n");

    // Example 8: Ignore default arguments
    println!("Example 8: Ignore default arguments");
    println!("------------------------------------");
    let mut ignore_args = Vec::new();
    ignore_args.push("--disable-dev-shm-usage".to_string());

    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .ignore_default_args(ignore_args)
        .build()
        .unwrap();

    let browser = playwright.chromium().launch(options).await?;
    println!("✓ Browser launched with filtered default args");
    println!("  Ignored: --disable-dev-shm-usage");
    browser.close().await?;
    println!("✓ Browser closed\n");

    // Example 9: Channel selection (if available)
    println!("Example 9: Channel selection");
    println!("----------------------------");
    println!("  Available channels: chrome, chrome-beta, chrome-dev, chrome-canary");
    println!("                      msedge, msedge-beta, msedge-dev");
    println!("  (Attempting to launch chrome channel)");

    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .channel("chrome".to_string())
        .build()
        .unwrap();

    match playwright.chromium().launch(options).await {
        Ok(browser) => {
            let version = browser.version().await?;
            println!("✓ Launched chrome channel successfully");
            println!("  Version: {}", version);
            browser.close().await?;
            println!("✓ Browser closed");
        }
        Err(e) => {
            println!("  Note: chrome channel not found ({})", e);
            println!("  This is expected if Chrome isn't installed in standard locations");
        }
    }

    println!("\n=== Demo completed successfully! ===");

    Ok(())
}
