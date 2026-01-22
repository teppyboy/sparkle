# Storage State Feature

This document describes the storage state feature in Sparkle, which provides Playwright-compatible API for saving and loading browser context state (cookies, localStorage, sessionStorage).

## Overview

The storage state feature allows you to:
- Save authentication state after login
- Restore session data across browser instances
- Share browser state between test runs
- Persist user preferences and settings

This implementation is fully compatible with Playwright Python's storage state format, allowing you to share state files between Playwright Python and Sparkle.

## API Reference

### Saving Storage State

```rust
use sparkle::async_api::Playwright;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let playwright = Playwright::new().await?;
    let browser = playwright.chromium().launch(Default::default()).await?;
    let context = browser.new_context(Default::default()).await?;
    let page = context.new_page().await?;
    
    // Perform login or other actions that set cookies/storage
    page.goto("https://example.com", Default::default()).await?;
    
    // Save storage state to a file
    let state = context.storage_state(Some("auth.json")).await?;
    println!("Saved {} cookies and {} origins", 
        state.cookies.len(), state.origins.len());
    
    browser.close().await?;
    Ok(())
}
```

### Loading Storage State from File

```rust
use sparkle::async_api::Playwright;
use sparkle::core::BrowserContextOptionsBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let playwright = Playwright::new().await?;
    let browser = playwright.chromium().launch(Default::default()).await?;
    
    // Create context with saved storage state
    let options = BrowserContextOptionsBuilder::default()
        .storage_state("auth.json")
        .build()
        .unwrap();
    
    let context = browser.new_context(options).await?;
    let page = context.new_page().await?;
    
    // Cookies and localStorage are already restored
    page.goto("https://example.com", Default::default()).await?;
    
    browser.close().await?;
    Ok(())
}
```

### Loading Storage State Inline

```rust
use sparkle::async_api::Playwright;
use sparkle::core::{BrowserContextOptionsBuilder, StorageState};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let playwright = Playwright::new().await?;
    let browser = playwright.chromium().launch(Default::default()).await?;
    
    // Load and modify storage state
    let mut state = StorageState::from_file("auth.json")?;
    // Modify cookies or storage as needed
    
    // Create context with inline storage state
    let options = BrowserContextOptionsBuilder::default()
        .storage_state(state)
        .build()
        .unwrap();
    
    let context = browser.new_context(options).await?;
    
    browser.close().await?;
    Ok(())
}
```

## Storage State Format

The storage state is saved as JSON with the following structure (compatible with Playwright):

```json
{
  "cookies": [
    {
      "name": "session",
      "value": "abc123",
      "domain": ".example.com",
      "path": "/",
      "expires": -1,
      "httpOnly": true,
      "secure": true,
      "sameSite": "Lax"
    }
  ],
  "origins": [
    {
      "origin": "https://example.com",
      "localStorage": [
        {
          "name": "username",
          "value": "john_doe"
        }
      ],
      "sessionStorage": []
    }
  ]
}
```

### Cookie Fields

- `name`: Cookie name (string)
- `value`: Cookie value (string)
- `domain`: Cookie domain (string, e.g., ".example.com")
- `path`: Cookie path (string, e.g., "/")
- `expires`: Expiration time in Unix seconds (float, -1 for session cookies)
- `httpOnly`: Whether the cookie is HTTP-only (boolean)
- `secure`: Whether the cookie is secure/HTTPS-only (boolean)
- `sameSite`: SameSite attribute ("Strict", "Lax", or "None")

### Origin Storage Fields

- `origin`: Full origin URL (string, e.g., "https://example.com")
- `localStorage`: Array of name-value pairs for localStorage
- `sessionStorage`: Array of name-value pairs for sessionStorage (optional, omitted if empty)

## Implementation Details

### Browser Support

The storage state feature is currently **Chromium-only**, as it relies on Chrome DevTools Protocol (CDP) for cookie management. Support includes:
- Chrome
- Chromium
- Microsoft Edge
- Other Chromium-based browsers

Firefox and WebKit are not currently supported.

### Origins Discovery

When saving storage state, only visited origins are captured. The implementation:
1. Iterates through all open pages in the context
2. Extracts the origin from each page's URL
3. Reads localStorage and sessionStorage for each unique origin
4. Skips origins that cannot be accessed or have already been processed

This matches Playwright's behavior where only origins with open pages have their storage captured.

### Storage Injection

When loading storage state into a new context:
1. Cookies are set first via CDP `Network.setCookies`
2. For each origin with storage data:
   - A temporary page is created and navigated to that origin
   - localStorage and sessionStorage are set via JavaScript execution
   - The temporary page is closed
3. All storage is applied before the context is returned to the user

### Error Handling

- File I/O errors (reading/writing storage state files) return `Error::ActionFailed`
- JSON parsing errors return `Error::ActionFailed` with descriptive messages
- CDP command failures return `Error::ActionFailed`
- Browser/context closed errors return `Error::BrowserClosed` or `Error::ContextClosed`

All errors are propagated via `Result<T>` and never panic.

## Testing

The storage state feature includes comprehensive unit tests:

```bash
# Run all storage tests
cargo test storage

# Run all tests
cargo test
```

Test coverage includes:
- JSON serialization/deserialization
- Playwright format compatibility
- All SameSite variants
- Empty storage states
- Session cookie handling
- localStorage and sessionStorage

## Examples

See `examples/storage_state.rs` for a complete working example:

```bash
cargo run --example storage_state --release
```

## Comparison with Playwright Python

This implementation closely matches Playwright Python's API:

### Playwright Python
```python
# Save storage state
state = await context.storage_state(path="auth.json")

# Load storage state
context = await browser.new_context(storage_state="auth.json")
# or
context = await browser.new_context(storage_state=state)
```

### Sparkle (Rust)
```rust
// Save storage state
let state = context.storage_state(Some("auth.json")).await?;

// Load storage state
let options = BrowserContextOptionsBuilder::default()
    .storage_state("auth.json")
    .build()
    .unwrap();
let context = browser.new_context(options).await?;
// or
let options = BrowserContextOptionsBuilder::default()
    .storage_state(state)
    .build()
    .unwrap();
let context = browser.new_context(options).await?;
```

The main differences are due to Rust's type system (using builders for options) and error handling (using `Result`).

## Future Enhancements

Potential future improvements:
- Add Firefox/WebKit support (if WebDriver/CDP allows)
- Add option to capture storage from specific origins
- Add option to merge storage states
- Add validation/sanitization utilities
- Support for IndexedDB and other storage mechanisms
