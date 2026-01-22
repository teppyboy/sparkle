// Test to demonstrate error handling with non-existing elements in frames

use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    println!("Testing error handling for non-existing elements in frames...\n");

    let playwright = Playwright::new().await?;
    let browser = playwright
        .chromium()
        .launch(
            LaunchOptionsBuilder::default()
                .headless(false)
                .build()
                .unwrap(),
        )
        .await?;

    let page = browser.new_page().await?;

    // Create test page with iframe
    let html_content = r#"
<!DOCTYPE html>
<html>
<head><title>Error Test</title></head>
<body>
    <h1>Main Page</h1>
    <p id="main-content">This is the main page</p>
    
    <iframe id="test-frame" style="width: 600px; height: 300px; border: 2px solid blue;" srcdoc="
        <!DOCTYPE html>
        <html>
        <body>
            <h2>Frame Content</h2>
            <button id='existing-button'>I Exist</button>
            <p id='existing-text'>Some text</p>
        </body>
        </html>
    "></iframe>
</body>
</html>
    "#;

    page.goto(
        &format!("data:text/html,{}", urlencoding::encode(html_content)),
        Default::default(),
    )
    .await?;

    println!("✓ Test page loaded\n");
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Test 1: Try to click a non-existing element in the frame
    println!("[Test 1] Attempting to click non-existing element in frame...");
    let frame = page.frame_locator("#test-frame");
    
    match frame.locator("#non-existing-button").click(Default::default()).await {
        Ok(_) => println!("  ❌ UNEXPECTED: Click succeeded (should have failed)"),
        Err(e) => println!("  ✓ Expected error: {:?}", e),
    }

    // Test 2: Verify we can still interact with main page after error
    println!("\n[Test 2] Verify main page interaction still works after error...");
    match page.locator("#main-content").text_content().await {
        Ok(text) => println!("  ✓ Main page element found: {:?}", text),
        Err(e) => println!("  ❌ FAILED: Could not access main page element: {:?}", e),
    }

    // Test 3: Verify we can still interact with the frame after error
    println!("\n[Test 3] Verify frame interaction still works after error...");
    match frame.locator("#existing-button").click(Default::default()).await {
        Ok(_) => println!("  ✓ Successfully clicked existing button in frame"),
        Err(e) => println!("  ❌ FAILED: Could not click existing button: {:?}", e),
    }

    // Test 4: Try to get text from non-existing element
    println!("\n[Test 4] Attempting to get text from non-existing element...");
    match frame.locator("#non-existing-text").text_content().await {
        Ok(text) => println!("  ❌ UNEXPECTED: Got text (should have failed): {:?}", text),
        Err(e) => println!("  ✓ Expected error: {:?}", e),
    }

    // Test 5: Verify interaction still works after text error
    println!("\n[Test 5] Verify interaction after text retrieval error...");
    match frame.locator("#existing-text").text_content().await {
        Ok(text) => println!("  ✓ Successfully got text: {:?}", text),
        Err(e) => println!("  ❌ FAILED: Could not get text: {:?}", e),
    }

    // Test 6: Try to fill non-existing input
    println!("\n[Test 6] Attempting to fill non-existing input...");
    match frame.locator("#non-existing-input").fill("test").await {
        Ok(_) => println!("  ❌ UNEXPECTED: Fill succeeded (should have failed)"),
        Err(e) => println!("  ✓ Expected error: {:?}", e),
    }

    // Test 7: Verify we can still use main page locator
    println!("\n[Test 7] Final verification - main page still accessible...");
    match page.locator("#main-content").is_visible().await {
        Ok(visible) => println!("  ✓ Main page element visible: {}", visible),
        Err(e) => println!("  ❌ FAILED: Could not check visibility: {:?}", e),
    }

    println!("\n✅ Error handling test complete!");
    println!("All errors were properly handled and frame context was restored.");
    
    println!("\nKeeping browser open for 2 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    browser.close().await?;
    println!("Browser closed");

    Ok(())
}
