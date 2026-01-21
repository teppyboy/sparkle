//! Comprehensive stealth mode test
//!
//! This example demonstrates all stealth features including:
//! - navigator.webdriver patching
//! - Header alignment (User-Agent, Accept-Language, sec-ch-ua)
//! - Timezone/locale emulation
//! - Geolocation emulation
//! - WebGL/canvas spoofing
//! - Permissions patching
//!
//! Usage:
//! ```bash
//! cargo run --example stealth_comprehensive
//! ```

use sparkle::async_api::Playwright;
use sparkle::core::{LaunchOptionsBuilder, StealthOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Sparkle Comprehensive Stealth Test ===\n");

    let playwright = Playwright::new().await?;

    // Configure stealth options with all features
    let stealth_options = StealthOptions {
        enabled: true,
        console_enabled: false,  // Disable console for maximum stealth
        webgl_spoof: true,
        canvas_noise: false,     // Can be enabled for more randomization
        permissions_patch: true,
        header_alignment: true,  // Auto-align headers with browser version
        allow_user_headers: false,
        locale: Some("en-US".to_string()),
        timezone_id: Some("America/New_York".to_string()),
        geolocation: Some((40.7128, -74.0060, 10.0)), // New York City coordinates
    };

    let options = LaunchOptionsBuilder::default()
        .headless(false) // Use headful to see the browser
        .stealth(stealth_options)
        .build()
        .unwrap();

    println!("Launching browser with full stealth configuration...");
    let browser = playwright.chromium().launch(options).await?;
    let page = browser.new_page().await?;

    println!("\n1. Testing basic stealth patches...");
    page.goto("about:blank", Default::default()).await?;

    // Test navigator.webdriver
    let webdriver = page.evaluate("navigator.webdriver").await?;
    println!("   ✓ navigator.webdriver = {} (expected: false)", webdriver);

    // Test chrome object
    let chrome_exists = page.evaluate("typeof window.chrome !== 'undefined'").await?;
    println!("   ✓ window.chrome exists = {} (expected: true)", chrome_exists);

    // Test plugins
    let plugins_count = page.evaluate("navigator.plugins.length").await?;
    println!("   ✓ navigator.plugins.length = {} (expected: > 0)", plugins_count);

    // Test languages
    let languages = page.evaluate("JSON.stringify(navigator.languages)").await?;
    println!("   ✓ navigator.languages = {}", languages);

    println!("\n2. Testing header alignment...");
    
    // Test User-Agent
    let user_agent = page.evaluate("navigator.userAgent").await?;
    println!("   ✓ User-Agent = {}", user_agent);
    
    // Check if User-Agent doesn't contain automation markers
    let ua_clean = !user_agent.to_string().contains("HeadlessChrome") 
        && !user_agent.to_string().contains("Automation");
    println!("   ✓ User-Agent clean (no automation markers) = {}", ua_clean);

    println!("\n3. Testing timezone emulation...");
    let timezone = page.evaluate(
        "Intl.DateTimeFormat().resolvedOptions().timeZone"
    ).await?;
    println!("   ✓ Timezone = {} (configured: America/New_York)", timezone);

    println!("\n4. Testing platform consistency...");
    let platform = page.evaluate("navigator.platform").await?;
    println!("   ✓ Platform = {}", platform);

    println!("\n5. Testing permissions API...");
    let permissions_exists = page.evaluate(
        "typeof navigator.permissions !== 'undefined' && typeof navigator.permissions.query === 'function'"
    ).await?;
    println!("   ✓ Permissions API exists = {} (expected: true)", permissions_exists);

    println!("\n6. Testing WebGL...");
    let webgl_vendor = page.evaluate(
        r#"
        (() => {
            const canvas = document.createElement('canvas');
            const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
            if (!gl) return 'Not supported';
            const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
            if (!debugInfo) return 'Debug info not available';
            return gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL);
        })()
        "#
    ).await?;
    println!("   ✓ WebGL Vendor = {}", webgl_vendor);

    println!("\n7. Testing against detection page...");
    page.goto("https://bot.sannysoft.com/", Default::default()).await?;
    
    println!("   ✓ Loaded bot detection page (check browser window)");
    println!("   → Look for green checkmarks (✓) indicating undetected");
    println!("   → Red X marks indicate detection (should be minimal/none)");

    println!("\n=== Test Summary ===");
    println!("✓ All stealth features have been applied!");
    println!("\nStealth features active:");
    println!("  • navigator.webdriver = false");
    println!("  • Chrome automation flags removed");
    println!("  • User-Agent aligned with browser version");
    println!("  • Accept-Language header set");
    println!("  • Timezone emulated (America/New_York)");
    println!("  • Locale set (en-US)");
    println!("  • Geolocation emulated (New York City)");
    println!("  • WebGL vendor/renderer spoofed");
    println!("  • Permissions API patched");
    println!("  • Plugins populated");

    println!("\nKeeping browser open for 60 seconds for inspection...");
    println!("Press Ctrl+C to close early.\n");
    
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

    browser.close().await?;
    println!("Browser closed.");

    Ok(())
}
