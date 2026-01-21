//! Device emulation example
//!
//! This example demonstrates how to use Playwright device descriptors to emulate
//! mobile devices and tablets.

use sparkle::async_api::Playwright;
use sparkle::core::{LaunchOptionsBuilder, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    sparkle::core::init_logging();

    println!("Starting Sparkle device emulation demo...\n");

    // Create Playwright instance
    let playwright = Playwright::new().await?;

    // List all available devices
    println!("Listing all available devices:");
    let devices = playwright.list_devices().await?;
    println!("Found {} devices\n", devices.len());

    // Show first 10 devices as examples
    println!("First 10 devices:");
    for device_name in devices.iter().take(10) {
        println!("  - {}", device_name);
    }
    println!();

    // Get iPhone 12 device descriptor
    println!("Fetching iPhone 12 device descriptor...");
    let iphone12 = playwright
        .devices("iPhone 12")
        .await?
        .expect("iPhone 12 should exist");

    println!("iPhone 12 specs:");
    println!("  User Agent: {}", iphone12.user_agent);
    println!(
        "  Viewport: {}x{}",
        iphone12.viewport.width, iphone12.viewport.height
    );
    println!("  Device Scale Factor: {}", iphone12.device_scale_factor);
    println!("  Is Mobile: {}", iphone12.is_mobile);
    println!("  Has Touch: {}", iphone12.has_touch);
    println!(
        "  Default Browser: {}\n",
        iphone12.default_browser_type
    );

    // Launch browser with device emulation
    println!("Launching browser with iPhone 12 emulation...");
    let launch_options = LaunchOptionsBuilder::default()
        .headless(false)
        .build()
        .unwrap();

    let browser = playwright.chromium().launch(launch_options).await?;

    // Create context with iPhone 12 settings
    let context_options = iphone12.to_context_options();
    let context = browser.new_context(context_options).await?;
    let page = context.new_page().await?;

    // Navigate to a website
    println!("Navigating to example.com...");
    page.goto("https://example.com", Default::default())
        .await?;

    println!("Page loaded! Browser will stay open for 5 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Try another device - Pixel 5
    println!("\nSwitching to Pixel 5 emulation...");
    let pixel5 = playwright
        .devices("Pixel 5")
        .await?
        .expect("Pixel 5 should exist");

    println!("Pixel 5 specs:");
    println!("  User Agent: {}", pixel5.user_agent);
    println!(
        "  Viewport: {}x{}",
        pixel5.viewport.width, pixel5.viewport.height
    );
    println!("  Device Scale Factor: {}", pixel5.device_scale_factor);
    println!();

    let context_options = pixel5.to_context_options();
    let context2 = browser.new_context(context_options).await?;
    let page2 = context2.new_page().await?;

    page2
        .goto("https://example.com", Default::default())
        .await?;

    println!("Pixel 5 page loaded! Browser will stay open for 5 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Cleanup
    println!("\nClosing browser...");
    browser.close().await?;

    println!("Device emulation demo completed!");

    Ok(())
}
