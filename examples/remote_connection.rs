//! Remote WebDriver connection example
//!
//! This example demonstrates connecting to an existing WebDriver server
//! instead of launching a local browser.
//!
//! # Prerequisites
//! You need a running WebDriver server. You can start one with:
//! 
//! ```bash
//! # Option 1: Start ChromeDriver manually
//! chromedriver --port=9515
//! 
//! # Option 2: Use Selenium Grid
//! # docker run -d -p 4444:4444 selenium/standalone-chrome
//! ```
//!
//! # Usage
//! ```bash
//! cargo run --example remote_connection
//! ```

use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for debugging (optional)
    tracing_subscriber::fmt::init();

    println!("Starting Sparkle remote connection example...");

    // Create Playwright instance
    let playwright = Playwright::new().await?;
    println!("Playwright initialized");

    // Build connection options
    let options = ConnectOptionsBuilder::default()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    // Connect to remote WebDriver server
    // Default ChromeDriver runs on http://localhost:9515
    let endpoint = "http://localhost:9515";
    println!("Connecting to WebDriver at {}...", endpoint);
    
    let browser = match playwright.chromium().connect(endpoint, options).await {
        Ok(browser) => {
            println!("✓ Successfully connected to remote browser");
            browser
        }
        Err(e) => {
            eprintln!("✗ Failed to connect to WebDriver server at {}", endpoint);
            eprintln!("  Error: {}", e);
            eprintln!("\nMake sure ChromeDriver is running:");
            eprintln!("  chromedriver --port=9515");
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

    // Get the current URL
    let url = page.url().await?;
    println!("Current URL: {}", url);

    // Take a screenshot
    let screenshot = page.screenshot().await?;
    let screenshot_len = screenshot.len();
    std::fs::write("remote_example.png", screenshot)?;
    println!("Screenshot saved to remote_example.png ({} bytes)", screenshot_len);

    // Close the browser
    browser.close().await?;
    println!("Browser closed");

    println!("\n✓ Remote connection example completed successfully!");

    Ok(())
}
