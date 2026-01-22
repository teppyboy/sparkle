// Integration test for frame_locator functionality

use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    println!("Starting frame_locator integration test...\n");

    // Launch browser
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

    // Create a test HTML page with iframes
    let html_content = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Frame Locator Test</title>
</head>
<body>
    <h1>Main Page</h1>
    <p id="main-content">This is the main page content</p>
    
    <iframe id="frame1" style="width: 600px; height: 300px; border: 2px solid blue;" srcdoc="
        <!DOCTYPE html>
        <html>
        <body>
            <h2>Frame 1</h2>
            <button id='frame1-button'>Click Me in Frame 1</button>
            <input id='frame1-input' placeholder='Type here' />
            <p id='frame1-result'>No action yet</p>
        </body>
        </html>
    "></iframe>
    
    <iframe id="frame2" style="width: 600px; height: 300px; border: 2px solid green; margin-top: 20px;" srcdoc="
        <!DOCTYPE html>
        <html>
        <body>
            <h2>Frame 2</h2>
            <div role='button' id='frame2-role-button'>Role Button</div>
            <input data-testid='test-input' placeholder='Test ID input' />
            <p id='frame2-content'>Initial text</p>
        </body>
        </html>
    "></iframe>
    
    <iframe id="nested-frame-parent" style="width: 600px; height: 400px; border: 2px solid red; margin-top: 20px;" srcdoc="
        <!DOCTYPE html>
        <html>
        <body>
            <h2>Parent Frame</h2>
            <p>This frame contains a nested iframe</p>
            <iframe id='nested-frame-child' style='width: 500px; height: 200px; border: 2px solid orange;' srcdoc='
                <!DOCTYPE html>
                <html>
                <body>
                    <h3>Nested Child Frame</h3>
                    <button id=nested-button>Click Me in Nested Frame</button>
                    <p id=nested-result>No click yet</p>
                </body>
                </html>
            '></iframe>
        </body>
        </html>
    "></iframe>
    
    <script>
        // Add click handlers for buttons in frames
        setTimeout(() => {
            const frame1 = document.getElementById('frame1');
            const frame1Doc = frame1.contentDocument;
            if (frame1Doc) {
                const btn = frame1Doc.getElementById('frame1-button');
                btn.addEventListener('click', () => {
                    frame1Doc.getElementById('frame1-result').textContent = 'Button clicked!';
                });
            }
            
            const nestedParent = document.getElementById('nested-frame-parent');
            const nestedParentDoc = nestedParent.contentDocument;
            if (nestedParentDoc) {
                const nestedChild = nestedParentDoc.getElementById('nested-frame-child');
                const nestedChildDoc = nestedChild.contentDocument;
                if (nestedChildDoc) {
                    const btn = nestedChildDoc.getElementById('nested-button');
                    btn.addEventListener('click', () => {
                        nestedChildDoc.getElementById('nested-result').textContent = 'Nested clicked!';
                    });
                }
            }
        }, 500);
    </script>
</body>
</html>
    "#;

    // Navigate to data URL with the test HTML
    page.goto(
        &format!("data:text/html,{}", urlencoding::encode(html_content)),
        Default::default(),
    )
    .await?;

    println!("✓ Test page loaded");
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // Test 1: Basic frame locator - click button in frame1
    println!("\n[Test 1] Click button in frame1");
    let frame1 = page.frame_locator("#frame1");
    frame1
        .locator("#frame1-button")
        .click(Default::default())
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let result_text = frame1.locator("#frame1-result").inner_text().await?;
    assert_eq!(result_text, "Button clicked!");
    println!("✓ Button click in frame1 successful: '{}'", result_text);

    // Test 2: Fill input in frame1
    println!("\n[Test 2] Fill input in frame1");
    frame1.locator("#frame1-input").fill("Hello Frame 1!").await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let input_value = frame1
        .locator("#frame1-input")
        .get_attribute("value")
        .await?;
    assert_eq!(input_value, Some("Hello Frame 1!".to_string()));
    println!("✓ Input filled in frame1: '{:?}'", input_value);

    // Test 3: get_by_role in frame2
    println!("\n[Test 3] Use get_by_role in frame2");
    let frame2 = page.frame_locator("#frame2");
    let role_button = frame2.get_by_role("button");
    let is_visible = role_button.is_visible().await?;
    println!("✓ Role button visible: {}", is_visible);
    assert!(is_visible);

    // Test 4: get_by_test_id in frame2
    println!("\n[Test 4] Use get_by_test_id in frame2");
    frame2.get_by_test_id("test-input").fill("Test data").await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let test_input_value = frame2
        .get_by_test_id("test-input")
        .get_attribute("value")
        .await?;
    assert_eq!(test_input_value, Some("Test data".to_string()));
    println!("✓ Test ID input filled: '{:?}'", test_input_value);

    // Test 5: Nested frame locator
    println!("\n[Test 5] Interact with nested frame");
    let parent_frame = page.frame_locator("#nested-frame-parent");
    let child_frame = parent_frame.frame_locator("#nested-frame-child");

    child_frame
        .locator("#nested-button")
        .click(Default::default())
        .await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let nested_result = child_frame.locator("#nested-result").inner_text().await?;
    assert_eq!(nested_result, "Nested clicked!");
    println!(
        "✓ Nested frame button click successful: '{}'",
        nested_result
    );

    // Test 6: get_by_text
    println!("\n[Test 6] Use get_by_text in frame1");
    let heading = frame1.get_by_text("Frame 1");
    let heading_text = heading.text_content().await?;
    println!("✓ Found heading by text: '{:?}'", heading_text);

    // Test 7: get_by_placeholder
    println!("\n[Test 7] Use get_by_placeholder in frame1");
    let placeholder_input = frame1.get_by_placeholder("Type here");
    placeholder_input.fill("Placeholder test").await?;
    let placeholder_value = placeholder_input.get_attribute("value").await?;
    assert_eq!(placeholder_value, Some("Placeholder test".to_string()));
    println!("✓ Placeholder input filled: '{:?}'", placeholder_value);

    println!("\n✅ All tests passed!");
    println!("\nKeeping browser open for 3 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    browser.close().await?;
    println!("Browser closed");

    Ok(())
}
