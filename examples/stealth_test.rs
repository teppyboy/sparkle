//! Test stealth mode functionality
//!
//! This example demonstrates stealth mode which makes automation undetectable.
//!
//! Usage:
//! ```bash
//! cargo run --example stealth_test
//! ```

use sparkle::async_api::Playwright;
use sparkle::core::{LaunchOptions, LaunchOptionsBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Sparkle Stealth Mode Test ===\n");

    let playwright = Playwright::new().await?;

    // Launch with stealth mode (enabled by default)
    println!("1. Testing stealth mode (default - enabled)...");
    let options = LaunchOptionsBuilder::default()
        .headless(false) // Use headful so we can see it
        .build()
        .unwrap();

    let browser = playwright.chromium().launch(options).await?;
    let page = browser.new_page().await?;

    // Navigate to a page that checks navigator.webdriver
    page.goto("about:blank", Default::default()).await?;

    // Check if navigator.webdriver is false (stealth working)
    let webdriver_value = page.evaluate("navigator.webdriver").await?;
    println!("   navigator.webdriver = {}", webdriver_value);

    // Check if chrome object exists
    let chrome_exists = page.evaluate("typeof window.chrome !== 'undefined'").await?;
    println!("   window.chrome exists = {}", chrome_exists);

    // Check plugins
    let plugins_count = page.evaluate("navigator.plugins.length").await?;
    println!("   navigator.plugins.length = {}", plugins_count);

    // Check languages
    let languages = page.evaluate("JSON.stringify(navigator.languages)").await?;
    println!("   navigator.languages = {}", languages);

    println!("\n✓ Stealth mode test completed");
    println!("\nExpected results:");
    println!("  - navigator.webdriver should be false (not undefined)");
    println!("  - window.chrome should exist (true)");
    println!("  - navigator.plugins.length should be > 0");
    println!("  - navigator.languages should have values");

    // Keep browser open for 5 seconds so user can inspect
    println!("\nKeeping browser open for 5 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    browser.close().await?;
    println!("\nBrowser closed.");

    Ok(())
}
