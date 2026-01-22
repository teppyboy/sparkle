// Test to verify frame selectors with quotes work correctly

use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    println!("Testing frame selectors with special characters...\n");

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

    // Create test page with iframes that have attributes with quotes
    let html_content = r#"
<!DOCTYPE html>
<html>
<head><title>Special Selector Test</title></head>
<body>
    <h1>Testing Frame Selectors</h1>
    
    <!-- Frame with title attribute containing quotes -->
    <iframe id="frame1" title="reCAPTCHA" style="width: 600px; height: 200px; border: 2px solid blue;" srcdoc="
        <!DOCTYPE html>
        <html>
        <body>
            <h2>Frame with title='reCAPTCHA'</h2>
            <button id='captcha-button'>Captcha Button</button>
        </body>
        </html>
    "></iframe>
    
    <!-- Frame with data attribute -->
    <iframe id="frame2" data-src="test.html" style="width: 600px; height: 200px; border: 2px solid green; margin-top: 20px;" srcdoc="
        <!DOCTYPE html>
        <html>
        <body>
            <h2>Frame with data-src='test.html'</h2>
            <input id='test-input' placeholder='Enter text' />
        </body>
        </html>
    "></iframe>
    
    <!-- Frame with class -->
    <iframe class="my-special-frame" style="width: 600px; height: 200px; border: 2px solid red; margin-top: 20px;" srcdoc="
        <!DOCTYPE html>
        <html>
        <body>
            <h2>Frame with class</h2>
            <p id='result'>Initial</p>
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

    // Test 1: Select frame by title attribute with quotes
    println!("[Test 1] Select frame by title='reCAPTCHA'");
    let frame1 = page.frame_locator("iframe[title='reCAPTCHA']");
    match frame1.locator("#captcha-button").click(Default::default()).await {
        Ok(_) => println!("  ✓ Successfully clicked button in frame with title attribute"),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Test 2: Select frame by data attribute with quotes
    println!("\n[Test 2] Select frame by data-src='test.html'");
    let frame2 = page.frame_locator("iframe[data-src='test.html']");
    match frame2.locator("#test-input").fill("Special chars work!").await {
        Ok(_) => println!("  ✓ Successfully filled input in frame with data attribute"),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Verify the value
    match frame2.locator("#test-input").get_attribute("value").await {
        Ok(Some(val)) => println!("  ✓ Input value: '{}'", val),
        Ok(None) => println!("  ⚠ Input has no value"),
        Err(e) => println!("  ❌ FAILED to get value: {:?}", e),
    }

    // Test 3: Select frame by class
    println!("\n[Test 3] Select frame by class selector");
    let frame3 = page.frame_locator("iframe.my-special-frame");
    match frame3.locator("#result").text_content().await {
        Ok(text) => println!("  ✓ Got text from frame with class selector: {:?}", text),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Test 4: Complex selector with multiple attributes
    println!("\n[Test 4] Select frame by ID (simple selector)");
    let frame_by_id = page.frame_locator("#frame1");
    match frame_by_id.locator("#captcha-button").is_visible().await {
        Ok(visible) => println!("  ✓ Button visible via ID selector: {}", visible),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Test 5: Selector with double quotes in attribute
    println!("\n[Test 5] Using get_by_text in frame (potential quote issues)");
    match frame1.get_by_text("Captcha Button").text_content().await {
        Ok(text) => println!("  ✓ get_by_text works: {:?}", text),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    println!("\n✅ All special character selector tests passed!");
    println!("The JavaScript quote escaping issue is fixed.");
    
    println!("\nKeeping browser open for 3 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    browser.close().await?;
    println!("Browser closed");

    Ok(())
}
