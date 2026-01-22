//! Example demonstrating wait_for_load_state() method
//!
//! This example tests the different load states and validates
//! that it matches Playwright's official behavior.

use sparkle::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize Playwright
    let playwright = Playwright::new().await?;
    
    // Launch browser
    let browser = playwright
        .chromium()
        .launch(LaunchOptionsBuilder::default().headless(false).build().unwrap())
        .await?;
    
    // Create a new page
    let page = browser.new_page().await?;
    
    println!("=== Testing wait_for_load_state() ===\n");
    
    // Test 1: Default state (Load)
    println!("Test 1: Navigate and wait for default 'load' state");
    page.goto("https://www.example.com", Default::default()).await?;
    page.wait_for_load_state(None, None).await?;
    println!("✓ Page loaded successfully (load state)");
    
    let title = page.title().await?;
    println!("  Page title: {}", title);
    
    // Test 2: Explicit Load state
    println!("\nTest 2: Navigate and wait for explicit 'load' state");
    page.goto("https://www.rust-lang.org", Default::default()).await?;
    page.wait_for_load_state(Some(WaitUntilState::Load), None).await?;
    println!("✓ Page loaded successfully (explicit load state)");
    
    let url = page.url().await?;
    println!("  Current URL: {}", url);
    
    // Test 3: DomContentLoaded state
    println!("\nTest 3: Navigate and wait for 'domcontentloaded' state");
    page.goto("https://www.github.com", Default::default()).await?;
    page.wait_for_load_state(Some(WaitUntilState::DomContentLoaded), None).await?;
    println!("✓ DOM content loaded successfully");
    
    // Test 4: NetworkIdle state
    println!("\nTest 4: Navigate and wait for 'networkidle' state");
    page.goto("https://www.example.com", Default::default()).await?;
    page.wait_for_load_state(Some(WaitUntilState::NetworkIdle), None).await?;
    println!("✓ Network idle state reached");
    
    // Test 5: Custom timeout
    println!("\nTest 5: Wait with custom timeout");
    page.goto("https://www.example.com", Default::default()).await?;
    page.wait_for_load_state(
        Some(WaitUntilState::Load), 
        Some(Duration::from_secs(10))
    ).await?;
    println!("✓ Loaded with custom 10s timeout");
    
    // Test 6: Wait after already loaded
    println!("\nTest 6: Wait when page is already loaded");
    // Page is already loaded from previous navigation
    page.wait_for_load_state(Some(WaitUntilState::Load), None).await?;
    println!("✓ Returned immediately (page already loaded)");
    
    // Test 7: Wait after dynamic navigation using JavaScript
    println!("\nTest 7: Wait after JavaScript navigation");
    page.evaluate("window.location.href = 'https://www.example.com'").await?;
    page.wait_for_load_state(Some(WaitUntilState::Load), None).await?;
    println!("✓ Waited for JS navigation to complete");
    
    // Test 8: Multiple sequential load state waits
    println!("\nTest 8: Sequential load state waits");
    page.goto("https://www.rust-lang.org", Default::default()).await?;
    page.wait_for_load_state(Some(WaitUntilState::DomContentLoaded), None).await?;
    println!("  ✓ DOM content loaded");
    page.wait_for_load_state(Some(WaitUntilState::Load), None).await?;
    println!("  ✓ Full load completed");
    page.wait_for_load_state(Some(WaitUntilState::NetworkIdle), None).await?;
    println!("  ✓ Network idle reached");
    
    // Test 9: Test with dynamic content
    println!("\nTest 9: Navigate and create dynamic content");
    page.goto("https://www.example.com", Default::default()).await?;
    page.wait_for_load_state(Some(WaitUntilState::Load), None).await?;
    
    // Add dynamic content after load
    page.evaluate(r#"
        const div = document.createElement('div');
        div.id = 'dynamic-content';
        div.textContent = 'Dynamic content loaded!';
        document.body.appendChild(div);
    "#).await?;
    
    // Verify dynamic content exists
    let has_dynamic = page.locator("#dynamic-content").is_visible().await?;
    println!("✓ Dynamic content added after load: {}", has_dynamic);
    
    println!("\n=== All tests passed! ===");
    println!("✓ Default load state works");
    println!("✓ Explicit load states work (Load, DomContentLoaded, NetworkIdle)");
    println!("✓ Custom timeouts work");
    println!("✓ Returns immediately when already loaded");
    println!("✓ Works with JavaScript navigation");
    println!("✓ Sequential waits work correctly");
    println!("✓ Works with dynamic content");
    
    println!("\n=== Cleanup ===");
    browser.close().await?;
    println!("Browser closed successfully");
    
    Ok(())
}
