//! Example demonstrating Locator API and element interactions
//!
//! This example shows how to use Sparkle's Locator API for reliable
//! element interactions with auto-waiting.

use sparkle::prelude::*;

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
    
    println!("Navigating to example form...");
    page.goto("https://www.example.com", Default::default()).await?;
    
    // Example 1: Using Locators (recommended approach)
    println!("\n=== Example 1: Using Locators ===");
    let heading = page.locator("h1");
    let text = heading.text_content().await?;
    println!("Heading text: {}", text);
    
    // Example 2: Direct page methods (convenience wrappers)
    println!("\n=== Example 2: Direct page methods ===");
    let title = page.title().await?;
    println!("Page title: {}", title);
    
    // Example 3: Check element visibility
    println!("\n=== Example 3: Element visibility ===");
    let is_visible = page.is_visible("h1").await?;
    println!("Heading is visible: {}", is_visible);
    
    // Example 4: Multiple locator methods
    println!("\n=== Example 4: Locator chaining ===");
    let locator = page.locator("p").first();
    if let Ok(text) = locator.text_content().await {
        println!("First paragraph: {}", text);
    }
    
    // Example 5: Get page content
    println!("\n=== Example 5: Page content ===");
    let html = page.content().await?;
    println!("Page HTML length: {} characters", html.len());
    
    // Example 6: JavaScript evaluation
    println!("\n=== Example 6: JavaScript evaluation ===");
    let doc_title = page.evaluate("document.title").await?;
    println!("Document title via JS: {:?}", doc_title);
    
    // Example 7: Take a screenshot
    println!("\n=== Example 7: Screenshots ===");
    let screenshot = page.screenshot().await?;
    println!("Screenshot captured: {} bytes", screenshot.len());
    
    // If you want to test form interactions, uncomment this section
    // and navigate to a page with a form:
    
    /*
    page.goto("https://example.com/form", Default::default()).await?;
    
    // Fill input field
    page.fill("input[name='email']", "user@example.com").await?;
    
    // Type with delay
    page.r#type("input[name='username']", "myusername", 
        TypeOptionsBuilder::default()
            .delay(std::time::Duration::from_millis(100))
            .build()
            .unwrap()
    ).await?;
    
    // Click a button
    page.click("button[type='submit']", Default::default()).await?;
    
    // Wait for element to appear
    page.wait_for_selector(".success-message").await?;
    */
    
    println!("\n=== Cleanup ===");
    browser.close().await?;
    println!("Browser closed successfully");
    
    Ok(())
}
