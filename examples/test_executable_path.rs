//! Test executable_path() function
//!
//! This example demonstrates getting the browser executable path.

use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Testing executable_path() function...\n");

    // Create Playwright instance
    let playwright = Playwright::new().await?;

    // Test Chromium
    println!("=== Chromium ===");
    let chromium = playwright.chromium();
    match chromium.executable_path() {
        Ok(path) => {
            println!("Chromium executable found at:");
            println!("  {}", path);
            if std::path::Path::new(&path).exists() {
                println!("  ✓ File exists!");
                
                // Check file size
                if let Ok(metadata) = std::fs::metadata(&path) {
                    let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
                    println!("  ✓ File size: {:.2} MB", size_mb);
                }
            } else {
                println!("  ✗ File does not exist!");
            }
        }
        Err(e) => {
            println!("Error: {}", e);
            println!("\nMake sure Chrome is installed:");
            println!("  cargo run --release -- install chrome");
        }
    }

    // Test Firefox (not implemented)
    println!("\n=== Firefox ===");
    let firefox = playwright.firefox();
    match firefox.executable_path() {
        Ok(_) => println!("  Unexpected: Firefox path found (not implemented yet)"),
        Err(e) => println!("  Expected error: {}", e),
    }

    // Test WebKit (not implemented)
    println!("\n=== WebKit ===");
    let webkit = playwright.webkit();
    match webkit.executable_path() {
        Ok(_) => println!("  Unexpected: WebKit path found (not implemented yet)"),
        Err(e) => println!("  Expected error: {}", e),
    }

    println!("\n=== Environment Variable Override ===");
    println!("You can override the Chrome path using CHROME_PATH environment variable:");
    println!("  CHROME_PATH=/custom/path/chrome cargo run --example test_executable_path");

    Ok(())
}
