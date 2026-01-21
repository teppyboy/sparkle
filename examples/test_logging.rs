//! Simple example to test logging functionality
//!
//! Run with different log levels:
//! - SPARKLE_LOG_LEVEL=info cargo run --example test_logging
//! - SPARKLE_LOG_LEVEL=debug cargo run --example test_logging
//! - SPARKLE_LOG_LEVEL=trace cargo run --example test_logging

use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging from SPARKLE_LOG_LEVEL environment variable
    init_logging();

    println!("Testing Sparkle logging...");
    println!("Set SPARKLE_LOG_LEVEL environment variable to see logs");
    println!();

    // These log statements should appear based on the log level
    tracing::trace!("This is a TRACE message");
    tracing::debug!("This is a DEBUG message");
    tracing::info!("This is an INFO message");
    tracing::warn!("This is a WARN message");
    tracing::error!("This is an ERROR message");

    println!();
    println!("Launching browser to test real logging...");

    // Launch browser - this will trigger logging in browser_type.rs
    let playwright = Playwright::new().await?;
    let browser = playwright
        .chromium()
        .launch(LaunchOptionsBuilder::default().headless(true).build().unwrap())
        .await?;

    println!("Browser launched successfully");

    // Create page - this will trigger logging in browser.rs
    let page = browser.new_page().await?;
    println!("Page created");

    // Navigate - this will trigger logging in page.rs and webdriver_adapter.rs
    page.goto("https://example.com", Default::default()).await?;
    println!("Navigation completed");

    // Close browser - this will trigger logging in browser.rs
    browser.close().await?;
    println!("Browser closed");

    println!();
    println!("Test completed successfully!");
    println!("If you set SPARKLE_LOG_LEVEL, you should see detailed logs above");

    Ok(())
}
