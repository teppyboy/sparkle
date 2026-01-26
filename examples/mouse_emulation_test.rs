// Example: Using mouse emulation to click captcha checkboxes with human-like behavior

use sparkle::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    println!("Testing mouse emulation for captcha clicking...\n");

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

    // Create a test page with a simulated Cloudflare challenge
    let html_content = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Mouse Emulation Test</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            padding: 20px;
        }
        .challenge-box {
            width: 300px;
            height: 78px;
            border: 2px solid #ccc;
            padding: 15px;
            background: #f9f9f9;
            border-radius: 3px;
        }
        .checkbox-container {
            display: flex;
            align-items: center;
            gap: 10px;
        }
        #captcha-checkbox {
            width: 28px;
            height: 28px;
            cursor: pointer;
        }
        #status {
            margin-top: 20px;
            padding: 10px;
            background: #e8f4f8;
            border-radius: 3px;
            font-size: 14px;
        }
        .event-log {
            margin-top: 20px;
            border: 1px solid #ddd;
            padding: 10px;
            background: #fafafa;
            max-height: 200px;
            overflow-y: auto;
            font-family: monospace;
            font-size: 12px;
        }
    </style>
</head>
<body>
    <h1>Mouse Emulation Test</h1>
    <p>This page tracks mouse events to detect human-like behavior</p>
    
    <div class="challenge-box">
        <div class="checkbox-container">
            <input type="checkbox" id="captcha-checkbox" />
            <label for="captcha-checkbox">I'm not a robot</label>
        </div>
    </div>
    
    <div id="status">Status: Waiting for interaction...</div>
    
    <div class="event-log" id="event-log">
        <strong>Event Log:</strong><br>
    </div>
    
    <script>
        const checkbox = document.getElementById('captcha-checkbox');
        const status = document.getElementById('status');
        const eventLog = document.getElementById('event-log');
        
        let mouseMoves = 0;
        let lastMoveTime = Date.now();
        let moveTimings = [];
        
        function logEvent(msg) {
            const entry = document.createElement('div');
            entry.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
            eventLog.appendChild(entry);
            eventLog.scrollTop = eventLog.scrollHeight;
        }
        
        document.addEventListener('mousemove', (e) => {
            mouseMoves++;
            const now = Date.now();
            const timeSinceLastMove = now - lastMoveTime;
            lastMoveTime = now;
            moveTimings.push(timeSinceLastMove);
            
            if (mouseMoves % 5 === 0) {
                logEvent(`Mouse move #${mouseMoves} at (${e.clientX}, ${e.clientY})`);
            }
        });
        
        checkbox.addEventListener('mousedown', (e) => {
            logEvent(`MOUSEDOWN detected at (${e.clientX}, ${e.clientY})`);
        });
        
        checkbox.addEventListener('mouseup', (e) => {
            logEvent(`MOUSEUP detected at (${e.clientX}, ${e.clientY})`);
        });
        
        checkbox.addEventListener('click', (e) => {
            const avgTiming = moveTimings.length > 0 
                ? moveTimings.reduce((a, b) => a + b, 0) / moveTimings.length 
                : 0;
            
            logEvent(`CLICK detected! Total mouse moves: ${mouseMoves}`);
            logEvent(`Average time between moves: ${avgTiming.toFixed(1)}ms`);
            
            if (checkbox.checked) {
                status.innerHTML = `✅ <strong>Success!</strong> Checkbox clicked with ${mouseMoves} mouse movements`;
                status.style.background = '#d4edda';
                status.style.color = '#155724';
                
                // Analyze if behavior looks human-like
                if (mouseMoves >= 5 && avgTiming > 5 && avgTiming < 100) {
                    logEvent('✅ Behavior appears human-like!');
                } else if (mouseMoves === 0) {
                    logEvent('⚠️ No mouse movement detected - possible bot');
                } else {
                    logEvent('⚠️ Movement pattern may be suspicious');
                }
            }
        });
        
        // Reset if unchecked
        checkbox.addEventListener('change', (e) => {
            if (!checkbox.checked) {
                status.innerHTML = 'Status: Waiting for interaction...';
                status.style.background = '#e8f4f8';
                status.style.color = '#000';
                mouseMoves = 0;
                moveTimings = [];
            }
        });
    </script>
</body>
</html>
    "#;

    page.goto(
        &format!("data:text/html,{}", urlencoding::encode(html_content)),
        Default::default(),
    )
    .await?;

    println!("✓ Test page loaded\n");
    tokio::time::sleep(Duration::from_millis(1000)).await;

    // Example 1: Direct click (no mouse movement - would be detected as bot)
    println!("=== Example 1: Direct click (bot-like behavior) ===");
    let checkbox_direct = page.locator("#captcha-checkbox");
    match checkbox_direct.click(Default::default()).await {
        Ok(_) => println!("✓ Direct click succeeded (but may be detected as bot)"),
        Err(e) => println!("❌ Failed: {}", e),
    }
    
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Uncheck to reset
    checkbox_direct.click(Default::default()).await.ok();
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Example 2: Human-like click with mouse emulation
    println!("\n=== Example 2: Human-like click with mouse emulation ===");
    
    let mouse = page.mouse();
    let checkbox_element = page.locator("#captcha-checkbox").element().await?;
    
    // Configure human-like click options
    let click_options = MouseClickOptions {
        move_to_element: true,
        move_options: MoveOptions {
            steps: 15,              // Smooth movement with 15 steps
            step_delay_ms: 12,      // 12ms between steps (realistic speed)
            jitter: true,           // Add natural jitter
            bezier_curve: true,     // Use Bezier curve for smooth path
        },
        delay_before_ms: Some(80),  // Wait 80ms before clicking
        mousedown_duration_ms: Some(120), // Hold mouse down for 120ms
    };
    
    println!("Moving mouse with human-like behavior...");
    match mouse.click_element(&checkbox_element, click_options).await {
        Ok(_) => println!("✓ Human-like click succeeded!"),
        Err(e) => println!("❌ Failed: {}", e),
    }
    
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Example 3: Click with custom settings
    println!("\n=== Example 3: Custom mouse movement settings ===");
    
    // Uncheck first
    checkbox_direct.click(Default::default()).await.ok();
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    let custom_options = MouseClickOptions {
        move_to_element: true,
        move_options: MoveOptions {
            steps: 20,              // More steps = smoother
            step_delay_ms: 8,       // Faster movement
            jitter: true,
            bezier_curve: true,
        },
        delay_before_ms: Some(50),
        mousedown_duration_ms: Some(100),
    };
    
    let checkbox_element2 = page.locator("#captcha-checkbox").element().await?;
    println!("Using custom movement settings...");
    match mouse.click_element(&checkbox_element2, custom_options).await {
        Ok(_) => println!("✓ Custom click succeeded!"),
        Err(e) => println!("❌ Failed: {}", e),
    }

    println!("\n=== Recommendations for Cloudflare/reCAPTCHA ===");
    println!("✅ Use mouse emulation with these settings:");
    println!("   - steps: 10-20 (smooth but not too slow)");
    println!("   - step_delay_ms: 8-15ms (human-like speed)");
    println!("   - jitter: true (natural mouse wobble)");
    println!("   - bezier_curve: true (natural curved path)");
    println!("   - delay_before_ms: 50-150ms (thinking time)");
    println!("   - mousedown_duration_ms: 50-150ms (realistic click)");
    println!();
    println!("❌ Avoid:");
    println!("   - Direct .click() without mouse movement");
    println!("   - Perfectly straight line movements");
    println!("   - Too fast movements (< 5ms per step)");
    println!("   - No delay before clicking");

    println!("\nKeeping browser open for 5 seconds to see the results...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    browser.close().await?;
    println!("Browser closed");

    Ok(())
}
