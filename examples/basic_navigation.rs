//! Basic navigation example
//!
//! This example demonstrates basic browser automation:
//! - Launching a Chromium browser
//! - Creating a page
//! - Navigating to a URL
//! - Getting page title
//! - Taking a screenshot
//! - Closing the browser
//!
//! # Prerequisites
//! ChromeDriver will be automatically launched from the installed location.
//! 
//! Install ChromeDriver if not already installed:
//! ```bash
//! sparkle install chrome
//! ```
//!
//! # Custom ChromeDriver
//! You can optionally specify a custom ChromeDriver:
//! - Set `CHROMEDRIVER_PATH` environment variable to use a specific ChromeDriver executable
//! - Set `CHROMEDRIVER_URL` environment variable to connect to an already-running ChromeDriver
//! 
//! Example:
//! ```bash
//! export CHROMEDRIVER_PATH=/path/to/chromedriver
//! # or
//! export CHROMEDRIVER_URL=http://localhost:9515
//! ```

use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for debugging (optional)
    tracing_subscriber::fmt::init();

    println!("Starting Sparkle basic navigation example...");

    // Create Playwright instance
    let playwright = Playwright::new().await?;
    println!("✓ Playwright initialized");

    // Build launch options
    let options = LaunchOptionsBuilder::default()
        .headless(true)  // Run in headless mode
        .build()
        .unwrap();

    // Launch Chromium browser
    let browser = playwright.chromium().launch(options).await?;
    println!("✓ Browser launched");

    // Create a new page
    let page = browser.new_page().await?;
    println!("✓ Page created");

    // Navigate to example.com
    let navigation_opts = NavigationOptionsBuilder::default()
        .build()
        .unwrap();
    
    page.goto("https://example.com", navigation_opts).await?;
    println!("✓ Navigated to example.com");

    // Get the page title
    let title = page.title().await?;
    println!("Page title: {}", title);

    // Get the current URL
    let url = page.url().await?;
    println!("Current URL: {}", url);

    // Take a screenshot
    let screenshot = page.screenshot().await?;
    let screenshot_len = screenshot.len();
    std::fs::write("example.png", screenshot)?;
    println!("✓ Screenshot saved to example.png ({} bytes)", screenshot_len);

    // Close the browser
    browser.close().await?;
    println!("✓ Browser closed");

    println!("\nExample completed successfully!");

    Ok(())
}
