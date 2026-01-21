//! CDP Session Example - Playwright-compatible API
//!
//! This example demonstrates using the Playwright-compatible CDPSession API
//! as well as Sparkle's convenience methods.

use serde_json::json;
use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    println!("=== Chrome DevTools Protocol Example ===\n");

    // Launch browser
    let playwright = Playwright::new().await?;
    let browser = playwright
        .chromium()
        .launch(LaunchOptionsBuilder::default().headless(true).build().unwrap())
        .await?;

    println!("✓ Browser launched\n");

    // ========================================================================
    // Method 1: Playwright-compatible API (recommended for portability)
    // ========================================================================
    println!("--- Method 1: Playwright-compatible API ---");
    println!("Using browser.new_browser_cdp_session() (matches Playwright)\n");

    let cdp_session = browser.new_browser_cdp_session().await?;
    println!("✓ CDPSession created");

    // Get browser version
    let version_info = cdp_session.send("Browser.getVersion", None).await?;
    println!("\n1. Browser version via CDPSession.send():");
    println!("   {}", serde_json::to_string_pretty(&version_info)?);

    // Evaluate JavaScript
    let params = json!({
        "expression": "navigator.userAgent",
        "returnByValue": true
    });
    let eval_result = cdp_session.send("Runtime.evaluate", Some(params)).await?;
    if let Some(value) = eval_result.get("result").and_then(|r| r.get("value")) {
        println!("\n2. JavaScript evaluation result:");
        println!("   User Agent: {}", value.as_str().unwrap_or("N/A"));
    }

    // Get performance metrics
    let metrics = cdp_session.send("Performance.getMetrics", None).await?;
    println!("\n3. Performance metrics:");
    if let Some(metrics_array) = metrics.get("metrics").and_then(|m| m.as_array()) {
        println!("   Total metrics: {}", metrics_array.len());
    }

    println!("\n✓ Playwright-compatible API completed\n");

    // ========================================================================
    // Method 2: Sparkle convenience methods (not in Playwright)
    // ========================================================================
    println!("--- Method 2: Sparkle Convenience Methods ---");
    println!("Using browser.execute_cdp() (Sparkle extension, NOT in Playwright)\n");

    // Direct CDP command
    let version_info2 = browser.execute_cdp("Browser.getVersion").await?;
    println!("1. Browser version via execute_cdp():");
    println!("   {}", serde_json::to_string_pretty(&version_info2)?);

    // CDP command with parameters
    let params2 = json!({
        "expression": "document.title || 'No title'",
        "returnByValue": true
    });
    let eval_result2 = browser
        .execute_cdp_with_params("Runtime.evaluate", params2)
        .await?;
    println!("\n2. JavaScript evaluation result:");
    if let Some(value) = eval_result2.get("result").and_then(|r| r.get("value")) {
        println!("   Title: {}", value.as_str().unwrap_or("N/A"));
    }

    println!("\n✓ Sparkle convenience methods completed\n");

    // ========================================================================
    // Comparison
    // ========================================================================
    println!("--- API Comparison ---\n");
    println!("Playwright-compatible (Method 1):");
    println!("  let cdp = browser.new_browser_cdp_session().await?;");
    println!("  let result = cdp.send(\"Browser.getVersion\", None).await?;");
    println!();
    println!("Sparkle convenience (Method 2):");
    println!("  let result = browser.execute_cdp(\"Browser.getVersion\").await?;");
    println!();
    println!("Both methods:");
    println!("  • Use the same stored CDP instance (efficient)");
    println!("  • Support all CDP commands");
    println!("  • Return JSON values");
    println!();
    println!("Choose Method 1 for Playwright compatibility");
    println!("Choose Method 2 for brevity (Sparkle-specific)");

    // Close browser
    browser.close().await?;
    println!("\n✓ Browser closed");

    println!("\n=== Example completed successfully! ===");

    Ok(())
}
