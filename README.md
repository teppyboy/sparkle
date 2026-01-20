# Sparkle

A Playwright implementation written in Rust, powered by `thirtyfour`.

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

## Overview

Sparkle provides a high-level, type-safe API for browser automation in Rust, closely following the Playwright Python API while being idiomatic to Rust. It's built on top of the WebDriver protocol using the `thirtyfour` crate and leverages `tokio` for async operations.

## Features

- ✅ **Async/Await**: Full async support using Tokio
- ✅ **Type-Safe**: Builder patterns with compile-time checking
- ✅ **Error Handling**: Comprehensive error types with Result pattern
- ✅ **Locator API**: Auto-waiting and retrying element selectors
- ✅ **Element Interactions**: Click, fill, type, and more
- ✅ **CLI Tool**: Download and manage browsers automatically
- ✅ **Browser Support**: Chromium/Chrome (Firefox and WebKit planned)
- ✅ **Rust-Idiomatic**: Following Rust best practices and conventions
- 🚧 **Full API Parity**: Implementing complete Playwright API (in progress)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
sparkle = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Prerequisites

Sparkle provides a CLI tool to automatically download and install Chrome and ChromeDriver:

### Option 1: Automatic Installation (Recommended)

```bash
# Install Sparkle CLI
cargo install --path .

# Install Chrome and ChromeDriver
sparkle install chromium

# Or install everything
sparkle install all
```

### Option 2: Manual Installation

If you prefer to install manually:

1. Download ChromeDriver from https://chromedriver.chromium.org/downloads
2. Run it:
   ```bash
   chromedriver --port=9515
   ```

Alternatively, set the `CHROMEDRIVER_URL` environment variable to point to your WebDriver endpoint.

## Quick Start

```rust
use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Create Playwright instance
    let playwright = Playwright::new().await?;
    
    // Launch browser
    let browser = playwright.chromium()
        .launch(Default::default())
        .await?;
    
    // Create a new page
    let page = browser.new_page().await?;
    
    // Navigate to a URL
    page.goto("https://example.com", Default::default()).await?;
    
    // Interact with elements using Locators (recommended)
    let button = page.locator("button#submit");
    button.click(Default::default()).await?;
    
    // Or use direct page methods (convenience wrappers)
    page.fill("input[name='email']", "user@example.com").await?;
    
    // Get element text
    let text = page.locator("h1").text_content().await?;
    println!("Heading: {}", text);
    
    // Get page title
    println!("Title: {}", page.title().await?);
    
    // Take a screenshot
    let screenshot = page.screenshot().await?;
    std::fs::write("screenshot.png", screenshot)?;
    
    // Close browser
    browser.close().await?;
    
    Ok(())
}
```

## Examples

See the [`examples/`](examples/) directory for more examples:

- [`basic_navigation.rs`](examples/basic_navigation.rs) - Basic navigation and screenshots
- [`locator_demo.rs`](examples/locator_demo.rs) - Using Locators for element interactions

Run an example:
```bash
# Make sure ChromeDriver is running first
cargo run --example basic_navigation
cargo run --example locator_demo
```

## API Comparison with Playwright Python

### Playwright Python
```python
from playwright.async_api import async_playwright

async with async_playwright() as playwright:
    browser = await playwright.chromium.launch(headless=False)
    page = await browser.new_page()
    await page.goto("https://example.com")
    await browser.close()
```

### Sparkle (Rust)
```rust
use sparkle::prelude::*;

let playwright = Playwright::new().await?;
let options = LaunchOptionsBuilder::default()
    .headless(false)
    .build()?;
let browser = playwright.chromium().launch(options).await?;
let page = browser.new_page().await?;
page.goto("https://example.com", Default::default()).await?;
browser.close().await?;
```

## CLI Tool

Sparkle includes a command-line tool for managing browser installations:

### Installation

```bash
# Install the Sparkle CLI
cargo install --path .

# Or build it locally
cargo build --bin sparkle --release
```

### Commands

**Install browsers:**
```bash
# Install Chrome and ChromeDriver
sparkle install chromium

# Install with all dependencies
sparkle install all

# Force reinstall
sparkle install chromium --force
```

**List installed browsers:**
```bash
sparkle list
```

**Uninstall browsers:**
```bash
# Uninstall Chrome
sparkle uninstall chrome

# Uninstall ChromeDriver
sparkle uninstall chromedriver

# Uninstall everything
sparkle uninstall all
```

### Installation Locations

Browsers are installed to platform-specific directories:
- **Windows**: `%APPDATA%\sparkle\browsers`
- **Linux**: `~/.local/share/sparkle/browsers`
- **macOS**: `~/Library/Application Support/com.sparkle.browsers`

The CLI automatically detects your platform and downloads the appropriate binaries from Google's Chrome for Testing repository.

## Architecture

```
sparkle/
├── async_api/           # Main async API (Browser, Page, etc.)
├── core/                # Core types (Error, Options, etc.)
└── driver/              # WebDriver adapter layer
```

### Key Components

- **Playwright**: Entry point for browser automation
- **BrowserType**: Manages browser types (Chromium, Firefox, WebKit)
- **Browser**: Represents a browser instance
- **BrowserContext**: Isolated browser context
- **Page**: Single tab/page in a browser
- **Locator**: Auto-waiting element selector (recommended)
- **ElementHandle**: Direct element reference
- **Options**: Builder-pattern configuration types

## Current Status

### Implemented ✅
- [x] Core error types and Result patterns
- [x] Playwright entry point
- [x] BrowserType with Chromium support
- [x] Browser and BrowserContext lifecycle
- [x] Page with basic navigation
- [x] Locator API (modern element selection with auto-waiting)
- [x] Element interactions (click, type, fill, text_content, etc.)
- [x] Screenshot capture
- [x] JavaScript evaluation
- [x] WebDriver adapter layer
- [x] Comprehensive option builders
- [x] **CLI tool for browser management**
  - [x] Automatic Chrome/ChromeDriver download
  - [x] Platform detection (Windows, Linux, macOS, ARM64)
  - [x] Install/uninstall commands
  - [x] Version management

### In Progress 🚧
- [ ] Network interception
- [ ] Frame handling
- [ ] Dialogs and file uploads
- [ ] Video recording
- [ ] Firefox and WebKit support

### Planned 📋
- [ ] Complete API parity with Playwright Python
- [ ] Integration tests with real browsers
- [ ] Performance optimizations
- [ ] Auto-start ChromeDriver from CLI-installed browsers

## Roadmap

See the detailed implementation plan in the project documentation. The implementation follows a phased approach:

1. **Phase 1**: Foundation (Core types, Browser lifecycle) - ✅ Complete
2. **Phase 2**: Page automation (Locators, element interactions) - ✅ Complete
3. **Phase CLI**: Browser installation CLI - ✅ Complete
4. **Phase 3**: Input devices (Keyboard, Mouse, Touch) - 🚧 Next
5. **Phase 4**: Advanced features (Network, Dialogs, Downloads)
6. **Phase 5**: Context features (Cookies, Storage, Emulation)
7. **Phase 6**: Frame support
8. **Phase 7**: API testing
9. **Phase 8**: Recording & Debugging
10. **Phase 9**: Multi-browser support

## Contributing

Contributions are welcome! This is an early-stage project and there's lots of work to do.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Acknowledgments

- [Playwright](https://playwright.dev/) - The original Playwright project by Microsoft
- [thirtyfour](https://github.com/stevepryde/thirtyfour) - Rust WebDriver client
- [tokio](https://tokio.rs/) - Async runtime for Rust

## Development

### Building

```bash
# Build the library
cargo build

# Build the CLI tool
cargo build --bin sparkle
```

### Running Tests

```bash
cargo test
```

### Using the CLI in Development

```bash
# Install browsers for development
cargo run --bin sparkle -- install chromium

# List installed browsers
cargo run --bin sparkle -- list

# Run examples with installed browsers
cargo run --example basic_navigation
```

### Running Examples

```bash
# Option 1: Use the CLI to install browsers first
cargo run --bin sparkle -- install chromium

# Option 2: Start ChromeDriver manually in another terminal
chromedriver --port=9515

# Then run examples
cargo run --example basic_navigation
cargo run --example locator_demo
```

## FAQ

**Q: Why not use thirtyfour directly?**

A: Sparkle provides a higher-level API that matches Playwright's design, with better ergonomics for common automation tasks, auto-waiting, and a more comprehensive feature set.

**Q: When will Firefox/WebKit be supported?**

A: Firefox support is planned for Phase 9. WebKit support depends on WebDriver availability and platform compatibility.

**Q: Can I use this in production?**

A: This is an early-stage project (v0.1). Core functionality works, but the API may change. Use with caution in production environments.

