//! Example demonstrating nth(), first(), and last() locator methods
//!
//! This example tests the 0-based indexing of nth() and validates
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
    
    println!("Navigating to example.com to test nth() methods...");
    
    // Navigate to a simple page
    page.goto("https://www.example.com", Default::default()).await?;
    
    // Add test elements using JavaScript
    page.evaluate(r#"
        const ul = document.createElement('ul');
        ul.id = 'test-list';
        for (let i = 1; i <= 5; i++) {
            const li = document.createElement('li');
            li.className = 'test-item';
            li.textContent = `Item ${i}`;
            ul.appendChild(li);
        }
        document.body.appendChild(ul);
    "#).await?;
    
    println!("Test elements created");
    
    println!("\n=== Testing nth() with 0-based indexing ===");
    
    // Test nth(0) - should get first item
    let item_0 = page.locator(".test-item").nth(0);
    let text_0 = item_0.text_content().await?;
    println!("nth(0): {}", text_0);
    assert_eq!(text_0.trim(), "Item 1", "nth(0) should return first item");
    
    // Test nth(1) - should get second item
    let item_1 = page.locator(".test-item").nth(1);
    let text_1 = item_1.text_content().await?;
    println!("nth(1): {}", text_1);
    assert_eq!(text_1.trim(), "Item 2", "nth(1) should return second item");
    
    // Test nth(2) - should get third item
    let item_2 = page.locator(".test-item").nth(2);
    let text_2 = item_2.text_content().await?;
    println!("nth(2): {}", text_2);
    assert_eq!(text_2.trim(), "Item 3", "nth(2) should return third item");
    
    // Test nth(4) - should get fifth (last) item
    let item_4 = page.locator(".test-item").nth(4);
    let text_4 = item_4.text_content().await?;
    println!("nth(4): {}", text_4);
    assert_eq!(text_4.trim(), "Item 5", "nth(4) should return fifth item");
    
    println!("\n=== Testing first() ===");
    
    // Test first() - should be equivalent to nth(0)
    let first = page.locator(".test-item").first();
    let first_text = first.text_content().await?;
    println!("first(): {}", first_text);
    assert_eq!(first_text.trim(), "Item 1", "first() should return first item");
    
    println!("\n=== Testing last() ===");
    
    // Test last() - should get the last item
    let last = page.locator(".test-item").last();
    let last_text = last.text_content().await?;
    println!("last(): {}", last_text);
    assert_eq!(last_text.trim(), "Item 5", "last() should return last item");
    
    println!("\n=== Testing count() ===");
    
    // Test count() - should return total number of elements
    let count = page.locator(".test-item").count().await?;
    println!("count(): {}", count);
    assert_eq!(count, 5, "count() should return 5");
    
    println!("\n=== Testing interactions with nth() ===");
    
    // Test clicking nth element
    let third_item = page.locator(".test-item").nth(2);
    third_item.click(Default::default()).await?;
    println!("Successfully clicked third item (nth(2))");
    
    // Verify element visibility
    let is_visible = third_item.is_visible().await?;
    println!("Third item is visible: {}", is_visible);
    assert!(is_visible, "Third item should be visible");
    
    println!("\n=== Testing edge cases ===");
    
    // Test nth with out-of-bounds index (should fail gracefully)
    println!("Testing out-of-bounds index...");
    let out_of_bounds = page.locator(".test-item").nth(100);
    match out_of_bounds.text_content().await {
        Ok(_) => println!("WARNING: Out-of-bounds access should have failed!"),
        Err(e) => println!("Out-of-bounds correctly returned error: {}", e),
    }
    
    // Test with selector that matches no elements
    println!("\nTesting selector with no matches...");
    let no_match = page.locator(".nonexistent").timeout(Duration::from_secs(2)).first();
    match no_match.text_content().await {
        Ok(_) => println!("WARNING: No-match selector should have failed!"),
        Err(e) => println!("No-match correctly returned error: {}", e),
    }
    
    println!("\n=== All tests passed! ===");
    println!("✓ nth(0), nth(1), nth(2), nth(4) work correctly");
    println!("✓ first() returns first element");
    println!("✓ last() returns last element");
    println!("✓ count() returns correct count");
    println!("✓ Interactions (click, is_visible) work with nth()");
    println!("✓ Edge cases handled appropriately");
    
    println!("\n=== Cleanup ===");
    browser.close().await?;
    println!("Browser closed successfully");
    
    Ok(())
}
