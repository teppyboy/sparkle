//! Wikipedia search example
//!
//! This example demonstrates:
//! - Launching a browser in headful mode (visible window)
//! - Navigating to Wikipedia
//! - Searching for "Playwright software"
//! - Clicking on the search button
//! - Clicking on the search result with "Playwright (software)" text
//! - Waiting 15 seconds before closing
//!
//! # Prerequisites
//! ChromeDriver will be automatically launched from the installed location.
//! 
//! Install ChromeDriver if not already installed:
//! ```bash
//! sparkle install chrome
//! ```

use sparkle::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing for debugging (optional)
    tracing_subscriber::fmt::init();

    println!("Starting Sparkle Wikipedia search example...");

    // Create Playwright instance
    let playwright = Playwright::new().await?;
    println!("Playwright initialized");

    // Build launch options - headful mode (visible browser window)
    let options = LaunchOptionsBuilder::default()
        .headless(false)  // Run in headful mode to see the browser
        .build()
        .unwrap();

    // Launch Chromium browser
    let browser = playwright.chromium().launch(options).await?;
    println!("Browser launched (headful mode)");

    // Get browser version
    let version = browser.version().await?;
    println!("Browser version: {}", version);

    // Create a new page
    let page = browser.new_page().await?;
    println!("Page created");

    // Navigate to Wikipedia (English)
    let navigation_opts = NavigationOptionsBuilder::default()
        .build()
        .unwrap();
    
    page.goto("https://en.wikipedia.org", navigation_opts).await?;
    println!("Navigated to Wikipedia (English)");

    // Wait a moment for the page to fully load
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Find the search input and type "Playwright software"
    let search_input = page.locator("#searchInput");
    search_input.fill("Playwright software").await?;
    println!("Typed 'Playwright software' into search box");

    // Wait a moment to see the typing
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Click the "Search" button using a simpler selector
    println!("Clicking search button...");
    // Try selecting any button inside the search form
    let search_button = page.locator("form#searchform button");
    search_button.click(Default::default()).await?;
    println!("Search submitted");

    // Wait for search results to load
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Get the current URL
    let url = page.url().await?;
    println!("Current URL: {}", url);
    
    // Check if we're on a search results page or directly on the article
    if url.contains("Special:Search") || url.contains("search=") {
        println!("On search results page, looking for 'Playwright (software)' result...");
        
        // Find the link containing "Playwright (software)" text
        // We'll find all search result links and look for the one with this text
        let playwright_software_link = page.locator("a[title*='Playwright (software)']");
        playwright_software_link.click(Default::default()).await?;
        println!("Clicked on Playwright (software) article");
        
        // Wait for article to load
        tokio::time::sleep(Duration::from_secs(2)).await;
    } else {
        println!("Directly navigated to article");
    }

    // Get the page title
    let title = page.title().await?;
    println!("Article title: {}", title);
    
    // Get final URL
    let final_url = page.url().await?;
    println!("Article URL: {}", final_url);

    // Wait 15 seconds before closing
    println!("\nWaiting 15 seconds before closing the browser...");
    println!("You can now see the Playwright article on Wikipedia!");
    tokio::time::sleep(Duration::from_secs(15)).await;

    // Close the browser
    browser.close().await?;
    println!("Browser closed");

    println!("\nExample completed successfully!");

    Ok(())
}
