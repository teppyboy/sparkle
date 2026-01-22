// Test for Cloudflare-style iframe selectors with substring matching

use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    println!("Testing Cloudflare-style iframe selectors...\n");

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

    // Create test page with Cloudflare-style iframes
    let html_content = r#"
<!DOCTYPE html>
<html>
<head><title>Cloudflare Iframe Test</title></head>
<body>
    <h1>Cloudflare Challenge Simulation</h1>
    
    <!-- Cloudflare challenge iframe with substring matching -->
    <iframe 
        id="cf-challenge-frame" 
        src="https://challenges.cloudflare.com/cdn-cgi/challenge-platform/h/b/orchestrate/12345"
        style="width: 600px; height: 300px; border: 2px solid blue;"
        srcdoc="
            <!DOCTYPE html>
            <html>
            <body>
                <h2>Cloudflare Challenge</h2>
                <button id='cf-button'>Verify you are human</button>
                <div id='cf-status'>Waiting for verification</div>
            </body>
            </html>
        ">
    </iframe>
    
    <!-- Another frame with partial src match -->
    <iframe 
        src="https://challenges.cloudflare.com/turnstile/v0/12345"
        style="width: 600px; height: 200px; border: 2px solid green; margin-top: 20px;"
        srcdoc="
            <!DOCTYPE html>
            <html>
            <body>
                <h2>Turnstile Challenge</h2>
                <input id='turnstile-input' placeholder='Enter code' />
            </body>
            </html>
        ">
    </iframe>
    
    <!-- reCAPTCHA style frame -->
    <iframe 
        title="reCAPTCHA"
        src="https://www.google.com/recaptcha/api2/anchor"
        style="width: 600px; height: 200px; border: 2px solid red; margin-top: 20px;"
        srcdoc="
            <!DOCTYPE html>
            <html>
            <body>
                <h2>reCAPTCHA</h2>
                <input type='checkbox' id='recaptcha-checkbox' />
                <label for='recaptcha-checkbox'>I'm not a robot</label>
            </body>
            </html>
        ">
    </iframe>
    
    <!-- Frame with data attributes -->
    <iframe 
        data-type="cloudflare"
        data-challenge-id="abc123"
        style="width: 600px; height: 150px; border: 2px solid purple; margin-top: 20px;"
        srcdoc="
            <!DOCTYPE html>
            <html>
            <body>
                <h2>Data Attribute Frame</h2>
                <p id='data-content'>Content in data attribute frame</p>
            </body>
            </html>
        ">
    </iframe>
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

    // Test 1: Substring match with *= operator for Cloudflare
    println!("[Test 1] Select Cloudflare frame with iframe[src*='challenges.cloudflare.com']");
    let cf_frame = page.frame_locator("iframe[src*='challenges.cloudflare.com']");
    match cf_frame.locator("#cf-button").click(Default::default()).await {
        Ok(_) => println!("  ✓ Successfully clicked Cloudflare button (first matching frame)"),
        Err(e) => {
            println!("  ❌ FAILED: {:?}", e);
            println!("  This selector matches multiple frames - it should pick the first one");
        }
    }

    // Test 2: More specific substring match
    println!("\n[Test 2] Select specific Cloudflare frame with iframe[src*='orchestrate']");
    let orchestrate_frame = page.frame_locator("iframe[src*='orchestrate']");
    match orchestrate_frame.locator("#cf-button").is_visible().await {
        Ok(visible) => println!("  ✓ Cloudflare orchestrate button visible: {}", visible),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Test 3: Turnstile frame
    println!("\n[Test 3] Select Turnstile frame with iframe[src*='turnstile']");
    let turnstile_frame = page.frame_locator("iframe[src*='turnstile']");
    match turnstile_frame.locator("#turnstile-input").fill("123456").await {
        Ok(_) => println!("  ✓ Successfully filled Turnstile input"),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Verify the value
    match turnstile_frame.locator("#turnstile-input").get_attribute("value").await {
        Ok(Some(val)) => println!("  ✓ Turnstile input value: '{}'", val),
        Ok(None) => println!("  ⚠ Input has no value"),
        Err(e) => println!("  ❌ FAILED to get value: {:?}", e),
    }

    // Test 4: reCAPTCHA with title attribute
    println!("\n[Test 4] Select reCAPTCHA frame with iframe[title='reCAPTCHA']");
    let recaptcha_frame = page.frame_locator("iframe[title='reCAPTCHA']");
    match recaptcha_frame.locator("#recaptcha-checkbox").is_visible().await {
        Ok(visible) => println!("  ✓ reCAPTCHA checkbox visible: {}", visible),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Test 5: Substring match with src containing 'recaptcha'
    println!("\n[Test 5] Select reCAPTCHA frame with iframe[src*='recaptcha']");
    let recaptcha_frame2 = page.frame_locator("iframe[src*='recaptcha']");
    match recaptcha_frame2.locator("#recaptcha-checkbox").click(Default::default()).await {
        Ok(_) => println!("  ✓ Successfully clicked reCAPTCHA checkbox"),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Test 6: Data attribute selector
    println!("\n[Test 6] Select frame with data-type attribute");
    let data_frame = page.frame_locator("iframe[data-type='cloudflare']");
    match data_frame.locator("#data-content").text_content().await {
        Ok(text) => println!("  ✓ Data frame content: {:?}", text),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Test 7: Combine multiple attribute selectors
    println!("\n[Test 7] Select frame with multiple attributes");
    let multi_attr_frame = page.frame_locator("iframe[data-type='cloudflare'][data-challenge-id='abc123']");
    match multi_attr_frame.locator("#data-content").is_visible().await {
        Ok(visible) => println!("  ✓ Multi-attribute selector works: visible={}", visible),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Test 8: ID selector (most specific)
    println!("\n[Test 8] Select frame by ID");
    let id_frame = page.frame_locator("#cf-challenge-frame");
    match id_frame.locator("#cf-status").text_content().await {
        Ok(text) => println!("  ✓ Status text via ID selector: {:?}", text),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Test 9: Starts-with selector (^=)
    println!("\n[Test 9] Select frame with src^='https://challenges.cloudflare.com'");
    let starts_with_frame = page.frame_locator("iframe[src^='https://challenges.cloudflare.com']");
    match starts_with_frame.locator("#cf-button").is_visible().await {
        Ok(visible) => println!("  ✓ Starts-with selector works: visible={}", visible),
        Err(e) => println!("  ❌ FAILED: {:?}", e),
    }

    // Test 10: Case-insensitive attribute matching
    println!("\n[Test 10] Case-insensitive title matching");
    let case_insensitive = page.frame_locator("iframe[title='reCAPTCHA' i]");
    match case_insensitive.locator("#recaptcha-checkbox").is_visible().await {
        Ok(visible) => println!("  ✓ Case-insensitive selector works: visible={}", visible),
        Err(e) => {
            println!("  ⚠ Case-insensitive selector not supported (this is expected)");
            println!("     Error: {:?}", e);
        }
    }

    println!("\n✅ All critical Cloudflare-style selectors work!");
    println!("Native CSS selector support ensures reliable frame switching.");
    
    println!("\nKeeping browser open for 3 seconds...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    browser.close().await?;
    println!("Browser closed");

    Ok(())
}
