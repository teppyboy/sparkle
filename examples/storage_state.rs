//! Storage State Example
//!
//! This example demonstrates how to save and load browser storage state
//! (cookies, localStorage, sessionStorage) using Sparkle, matching Playwright's API.
//!
//! Use cases:
//! - Save authentication state after login
//! - Restore session data across browser instances
//! - Share browser state between tests
//!
//! Run with: cargo run --example storage_state --release

use sparkle::async_api::Playwright;
use sparkle::core::{BrowserContextOptionsBuilder, LaunchOptionsBuilder, StorageState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    sparkle::core::init_logging();

    println!("=== Sparkle Storage State Example ===\n");

    // Create Playwright instance
    let playwright = Playwright::new().await?;

    // Launch browser
    let launch_options = LaunchOptionsBuilder::default()
        .headless(false)
        .build()
        .unwrap();

    let browser = playwright.chromium().launch(launch_options).await?;

    // Part 1: Create a session and save storage state
    println!("Part 1: Creating session and saving storage state...");
    {
        let context = browser.new_context(Default::default()).await?;
        let page = context.new_page().await?;

        // Navigate to a test page
        page.goto("https://example.com", Default::default()).await?;
        println!("Navigated to example.com");

        // Set some localStorage data
        page.evaluate(
            r#"
            localStorage.setItem('username', 'john_doe');
            localStorage.setItem('theme', 'dark');
            localStorage.setItem('session_id', 'abc123xyz');
            "#,
        )
        .await?;
        println!("Set localStorage items");

        // The page should have some cookies too (if example.com sets any)
        // For demonstration, we'll just work with what we have

        // Save storage state to a file
        let storage_state = context.storage_state(Some("state.json")).await?;
        println!("\nStorage state saved to state.json");
        println!("  - Cookies: {}", storage_state.cookies.len());
        println!("  - Origins: {}", storage_state.origins.len());

        if !storage_state.origins.is_empty() {
            for origin in &storage_state.origins {
                println!("    Origin: {}", origin.origin);
                println!("      localStorage: {} items", origin.local_storage.len());
                for item in &origin.local_storage {
                    println!("        - {}: {}", item.name, item.value);
                }
            }
        }

        context.close().await?;
    }

    println!("\n");

    // Part 2: Load storage state from file
    println!("Part 2: Loading storage state from file...");
    {
        // Create a new context with the saved storage state
        let context_options = BrowserContextOptionsBuilder::default()
            .storage_state("state.json")
            .build()
            .unwrap();

        let context = browser.new_context(context_options).await?;
        let page = context.new_page().await?;

        // Navigate to the same page
        page.goto("https://example.com", Default::default()).await?;
        println!("Navigated to example.com with restored storage");

        // Verify localStorage was restored
        let username = page
            .evaluate("return localStorage.getItem('username');")
            .await?;
        let theme = page
            .evaluate("return localStorage.getItem('theme');")
            .await?;
        let session_id = page
            .evaluate("return localStorage.getItem('session_id');")
            .await?;

        println!("\nRestored localStorage:");
        println!("  - username: {}", username);
        println!("  - theme: {}", theme);
        println!("  - session_id: {}", session_id);

        context.close().await?;
    }

    println!("\n");

    // Part 3: Use inline StorageState object
    println!("Part 3: Creating context with inline StorageState...");
    {
        // Load storage state from file into a StorageState object
        let storage_state = StorageState::from_file("state.json")?;

        // Modify it if needed
        let modified_state = storage_state.clone();
        // You can add/modify cookies or storage here

        // Create context with inline storage state
        let context_options = BrowserContextOptionsBuilder::default()
            .storage_state(modified_state)
            .build()
            .unwrap();

        let context = browser.new_context(context_options).await?;
        let page = context.new_page().await?;

        page.goto("https://example.com", Default::default()).await?;
        println!("Navigated with inline storage state");

        // Verify
        let username = page
            .evaluate("return localStorage.getItem('username');")
            .await?;
        println!("Verified username from inline state: {}", username);

        context.close().await?;
    }

    // Cleanup
    browser.close().await?;

    println!("\n=== Example completed successfully ===");
    println!("Storage state file 'state.json' has been created in the current directory.");
    println!("You can inspect it to see the Playwright-compatible JSON format.");

    Ok(())
}
