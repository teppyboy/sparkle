//! CDP (Chrome DevTools Protocol) connection example
//!
//! This example demonstrates connecting to a Chrome browser via CDP
//! using the --remote-debugging-port flag.
//!
//! # Prerequisites
//! You need Chrome running with remote debugging enabled:
//! 
//! ```bash
//! # Windows
//! chrome.exe --remote-debugging-port=9222 --user-data-dir=C:\tmp\chrome-debug
//! 
//! # macOS
//! /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome \
//!   --remote-debugging-port=9222 --user-data-dir=/tmp/chrome-debug
//! 
//! # Linux
//! google-chrome --remote-debugging-port=9222 --user-data-dir=/tmp/chrome-debug
//! ```
//!
//! Note: The --user-data-dir flag creates a separate profile to avoid conflicts
//! with your regular Chrome browsing.
//!
//! # Usage
//! ```bash
//! cargo run --example cdp_connection
//! ```

use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for debugging (optional)
    tracing_subscriber::fmt::init();

    println!("Starting Sparkle CDP connection example...");
    println!("\nNote: This example connects to Chrome via the Chrome DevTools Protocol.");
    println!("Make sure Chrome is running with --remote-debugging-port=9222\n");

    // Create Playwright instance
    let playwright = Playwright::new().await?;
    println!("Playwright initialized");

    // Build CDP connection options
    let options = ConnectOverCdpOptionsBuilder::default()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    // Connect via CDP
    // Chrome with --remote-debugging-port=9222 exposes WebDriver on the same port
    let endpoint = "http://localhost:9222";
    println!("Connecting to Chrome via CDP at {}...", endpoint);
    
    let browser = match playwright.chromium().connect_over_cdp(endpoint, options).await {
        Ok(browser) => {
            println!("✓ Successfully connected via CDP");
            browser
        }
        Err(e) => {
            eprintln!("✗ Failed to connect via CDP at {}", endpoint);
            eprintln!("  Error: {}", e);
            eprintln!("\nMake sure Chrome is running with remote debugging:");
            eprintln!("  chrome --remote-debugging-port=9222 --user-data-dir=/tmp/chrome-debug");
            return Err(e);
        }
    };

    // Get browser version
    let version = browser.version().await?;
    println!("Browser version: {}", version);

    // Create a new page
    let page = browser.new_page().await?;
    println!("Page created");

    // Navigate to example.com
    let navigation_opts = NavigationOptionsBuilder::default()
        .build()
        .unwrap();
    
    page.goto("https://example.com", navigation_opts).await?;
    println!("Navigated to example.com");

    // Get the page title
    let title = page.title().await?;
    println!("Page title: {}", title);

    // Execute JavaScript via CDP (using WebDriver adapter)
    let user_agent = page.evaluate("navigator.userAgent").await?;
    println!("User Agent: {}", user_agent);

    // Get the current URL
    let url = page.url().await?;
    println!("Current URL: {}", url);

    // Take a screenshot
    let screenshot = page.screenshot().await?;
    let screenshot_len = screenshot.len();
    std::fs::write("cdp_example.png", screenshot)?;
    println!("Screenshot saved to cdp_example.png ({} bytes)", screenshot_len);

    // Close the browser connection (browser process continues running)
    browser.close().await?;
    println!("Browser connection closed (browser process still running)");

    println!("\n✓ CDP connection example completed successfully!");
    println!("Note: The Chrome browser is still running. Close it manually or via Task Manager.");

    Ok(())
}
