# Advanced Stealth Features

This document describes the advanced stealth features implemented in Sparkle for undetectable browser automation.

## Overview

Sparkle includes **Patchright-style stealth mode** that makes Chromium automation undetectable by anti-bot systems. The stealth implementation includes:

1. **Browser Switch Patching** - Removes automation flags
2. **Navigator Property Spoofing** - Patches `navigator.webdriver`, plugins, languages
3. **Header Alignment** - Automatically aligns User-Agent and headers with browser version
4. **Timezone/Locale Emulation** - Consistent timezone and locale settings
5. **Geolocation Emulation** - Custom GPS coordinates
6. **WebGL/Canvas Spoofing** - Fingerprint randomization
7. **Permissions Patching** - Avoids detection via permissions API

## Configuration

### StealthOptions

```rust
use sparkle::core::StealthOptions;

let stealth = StealthOptions {
    enabled: true,                    // Enable stealth mode (default)
    console_enabled: false,           // Disable console for stealth (default)
    webgl_spoof: true,                // Spoof WebGL vendor/renderer
    canvas_noise: false,              // Add canvas noise (optional)
    permissions_patch: true,          // Patch permissions API
    header_alignment: true,           // Auto-align headers
    allow_user_headers: false,        // Block custom headers
    locale: Some("en-US".to_string()), // Set locale
    timezone_id: Some("America/New_York".to_string()), // Set timezone
    geolocation: Some((40.7128, -74.0060, 10.0)),     // NYC coordinates
};
```

### Launch with Stealth

```rust
use sparkle::async_api::Playwright;
use sparkle::core::LaunchOptionsBuilder;

let playwright = Playwright::new().await?;

let options = LaunchOptionsBuilder::default()
    .headless(false)
    .stealth(stealth)
    .build()
    .unwrap();

let browser = playwright.chromium().launch(options).await?;
```

**Note**: Stealth is **enabled by default** in Sparkle. You don't need to configure it unless you want to customize settings.

## Features in Detail

### 1. Browser Switch Patching

Automatically removes automation-related Chrome switches:
- Removes `--enable-automation`
- Adds `--disable-blink-features=AutomationControlled`
- Removes automation-related flags

**Result**: Browser doesn't report as automated.

### 2. Navigator Property Spoofing

JavaScript patches applied via CDP `Page.addScriptToEvaluateOnNewDocument`:

```javascript
// navigator.webdriver → false
Object.defineProperty(navigator, 'webdriver', {
    get: () => false
});

// window.chrome object
window.chrome = { runtime: {} };

// plugins populated with realistic values
navigator.plugins = [/* PDF viewers, etc. */];

// languages set to realistic values
navigator.languages = ['en-US', 'en'];
```

### 3. Header Alignment

Headers are automatically aligned with the actual browser version using CDP `Network.setUserAgentOverride`:

**User-Agent Example:**
```
Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 
(KHTML, like Gecko) Chrome/120.0.6099.129 Safari/537.36
```

**Accept-Language Example:**
```
en-US,en;q=0.9
```

**Sec-CH-UA Headers:**
```
sec-ch-ua: "Not_A Brand";v="8", "Chromium";v="120", "Google Chrome";v="120"
sec-ch-ua-mobile: ?0
sec-ch-ua-platform: "Windows"
```

### 4. Timezone Emulation

Sets timezone using CDP `Emulation.setTimezoneOverride`:

```rust
timezone_id: Some("America/New_York".to_string())
```

JavaScript will see the configured timezone:
```javascript
Intl.DateTimeFormat().resolvedOptions().timeZone
// → "America/New_York"
```

### 5. Locale Emulation

Sets browser locale using CDP `Emulation.setLocaleOverride`:

```rust
locale: Some("en-US".to_string())
```

Affects:
- Accept-Language header
- `navigator.language`
- Date/time formatting

### 6. Geolocation Emulation

Sets GPS coordinates using CDP `Emulation.setGeolocationOverride`:

```rust
// (latitude, longitude, accuracy_meters)
geolocation: Some((40.7128, -74.0060, 10.0))
```

JavaScript geolocation API will return configured coordinates:
```javascript
navigator.geolocation.getCurrentPosition(pos => {
    console.log(pos.coords.latitude);  // 40.7128
    console.log(pos.coords.longitude); // -74.0060
});
```

### 7. WebGL Spoofing

Patches WebGL vendor/renderer strings to avoid fingerprinting:

```javascript
// Before: "Google Inc. (NVIDIA)"
// After:  "Intel Inc."
```

### 8. Canvas Noise

Adds subtle noise to canvas operations to randomize fingerprints (optional):

```rust
canvas_noise: true
```

This makes canvas fingerprinting less reliable while keeping drawings visually identical.

## Testing Stealth

### Detection Test Sites

Test your stealth configuration against these sites:

1. **Bot.Sannysoft** - https://bot.sannysoft.com/
   - Tests common automation detection vectors
   - Look for green checkmarks (✓)

2. **AreYouHeadless** - https://arh.antoinevastel.com/bots/areyouheadless
   - Tests headless Chrome detection
   - Should show "You are not Chrome Headless"

3. **CreepJS** - https://abrahamjuliot.github.io/creepjs/
   - Comprehensive fingerprinting test
   - Check for consistent fingerprint

### Example Test Script

```rust
use sparkle::async_api::Playwright;
use sparkle::core::LaunchOptionsBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let playwright = Playwright::new().await?;
    
    // Stealth enabled by default!
    let browser = playwright.chromium().launch(Default::default()).await?;
    let page = browser.new_page().await?;
    
    // Test navigator.webdriver
    let webdriver = page.evaluate("navigator.webdriver").await?;
    assert_eq!(webdriver.to_string(), "false");
    
    // Test against detection site
    page.goto("https://bot.sannysoft.com/", Default::default()).await?;
    
    // Keep open for inspection
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    browser.close().await?;
    
    Ok(())
}
```

## Opt-Out

To disable stealth mode:

```rust
use sparkle::core::StealthOptions;

let stealth = StealthOptions {
    enabled: false,  // Disable all stealth features
    ..Default::default()
};

let options = LaunchOptionsBuilder::default()
    .stealth(stealth)
    .build()
    .unwrap();
```

## Limitations

### Chromium Only

Stealth mode is **Chromium-only**. Firefox and WebKit are not supported.

### Console API

By default, `console_enabled: false` disables the console API for better stealth. This means:
- `console.log()` won't output in browser console
- Use CDP for debugging instead

To enable console (reduces stealth):
```rust
console_enabled: true
```

### User Headers

Custom headers may leak automation. By default, user-provided headers are blocked when stealth is enabled:

```rust
allow_user_headers: false  // Default - blocks custom headers
```

To allow custom headers (reduces stealth):
```rust
allow_user_headers: true
```

## Implementation Details

### CDP Commands Used

Stealth mode uses the following Chrome DevTools Protocol commands:

1. `Page.addScriptToEvaluateOnNewDocument` - Injects stealth JS
2. `Network.setUserAgentOverride` - Sets User-Agent and headers
3. `Emulation.setTimezoneOverride` - Sets timezone
4. `Emulation.setLocaleOverride` - Sets locale
5. `Emulation.setGeolocationOverride` - Sets GPS coordinates

### Injection Timing

Stealth features are applied:
- **Before** any page content loads
- **On every navigation**
- **In all frames** (main frame and iframes)
- **Immediately** when page is created

This ensures automation is undetectable from the first moment.

## Comparison with Playwright

| Feature | Playwright | Sparkle |
|---------|-----------|---------|
| Default stealth | ❌ Disabled | ✅ Enabled |
| Switch patching | Manual | ✅ Automatic |
| Header alignment | Manual | ✅ Automatic |
| Console disabled | ❌ No | ✅ Yes (default) |
| Geolocation | Via context | ✅ Via stealth options |
| Timezone | Via context | ✅ Via stealth options |
| WebGL spoofing | ❌ No | ✅ Yes |

## Best Practices

### 1. Use Default Configuration

For most use cases, the default stealth configuration is sufficient:

```rust
// Stealth enabled automatically!
let browser = playwright.chromium().launch(Default::default()).await?;
```

### 2. Match Timezone to Geolocation

If setting geolocation, match the timezone:

```rust
// New York
timezone_id: Some("America/New_York".to_string()),
geolocation: Some((40.7128, -74.0060, 10.0)),

// London
timezone_id: Some("Europe/London".to_string()),
geolocation: Some((51.5074, -0.1278, 10.0)),
```

### 3. Use Realistic Locales

Match locale to expected user location:

```rust
// US user
locale: Some("en-US".to_string()),

// UK user
locale: Some("en-GB".to_string()),

// French user
locale: Some("fr-FR".to_string()),
```

### 4. Test Before Deployment

Always test against detection sites before deploying:
- https://bot.sannysoft.com/
- https://arh.antoinevastel.com/bots/areyouheadless
- https://abrahamjuliot.github.io/creepjs/

### 5. Keep Browser Updated

Stealth effectiveness depends on using recent Chrome versions. Keep your Chrome installation up to date:

```bash
sparkle install chrome
```

## Troubleshooting

### Still Being Detected?

1. **Check browser version** - Ensure you're using recent Chrome
2. **Test headers** - Verify User-Agent matches Chrome version
3. **Enable canvas noise** - Adds more fingerprint randomization
4. **Disable custom headers** - Set `allow_user_headers: false`
5. **Disable console** - Keep `console_enabled: false`

### Console Not Working?

Console is disabled by default for stealth. To enable:

```rust
console_enabled: true
```

Note: This may reduce stealth effectiveness.

### Timezone Not Applied?

Ensure you're setting timezone before navigation:

```rust
let page = browser.new_page().await?; // Stealth applied here
page.goto("https://example.com", Default::default()).await?;
```

## References

- **Patchright**: https://github.com/Kaliiiiiiiiii-Vinyzu/patchright
- **Chrome DevTools Protocol**: https://chromedevtools.github.io/devtools-protocol/
- **Playwright**: https://playwright.dev/

## Support

Stealth mode is an advanced feature. If you encounter issues:

1. Check the [examples/stealth_comprehensive.rs](../examples/stealth_comprehensive.rs) example
2. Test against detection sites
3. Report issues at https://github.com/your-org/sparkle/issues
