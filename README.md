# Sparkle

> A Playwright-like browser automation library for Rust, powered by WebDriver

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Sparkle brings Playwright's intuitive API to Rust with full async support, type safety, and automatic browser management.

## Features

- **Async/Await** - Built on Tokio for efficient async operations
- **Type-Safe** - Builder patterns with compile-time guarantees
- **Auto-Waiting Locators** - Modern element selection that handles timing
- **CLI Tool** - Automatic browser/driver download and management
- **Playwright Compatible** - Shares browser cache with Playwright installations
- **Zero Config** - ChromeDriver auto-launches and manages itself
- **Fast & Ergonomic** - Idiomatic Rust with minimal boilerplate


## Quick Start

### 1. Install Sparkle CLI

```bash
cargo install --path .
```

### 2. Download Browser

```bash
# Installs Chrome + ChromeDriver to Playwright's cache directory
sparkle install chrome
```

### 3. Write Your First Script

```rust
use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let playwright = Playwright::new().await?;
    let browser = playwright.chromium().launch(Default::default()).await?;
    let page = browser.new_page().await?;
    
    page.goto("https://example.com", Default::default()).await?;
    page.locator("button#submit").click(Default::default()).await?;
    
    let text = page.locator("h1").text_content().await?;
    println!("Heading: {}", text);
    
    browser.close().await?;
    Ok(())
}
```

### 4. Run It

```bash
cargo run --example your_script
```

ChromeDriver launches automatically - no manual setup required!


## CLI Commands

```bash
# Install Chrome + ChromeDriver (default)
sparkle install chrome

# Skip ChromeDriver installation
sparkle install chrome --skip-driver

# Force reinstall
sparkle install chrome --force

# List installed browsers
sparkle list

# Uninstall
sparkle uninstall chrome
sparkle uninstall all
```

**Installation Location:**  
Browsers install to Playwright's cache (`%LOCALAPPDATA%\ms-playwright` on Windows, `~/.cache/ms-playwright` on Linux, `~/Library/Caches/ms-playwright` on macOS). This allows sharing browsers with Playwright installations.

## Advanced Configuration

### Custom ChromeDriver

```bash
# Use custom ChromeDriver executable
export CHROMEDRIVER_PATH=/path/to/chromedriver

# Connect to existing ChromeDriver server
export CHROMEDRIVER_URL=http://localhost:9515

# Use custom Chrome binary
export CHROME_PATH=/path/to/chrome
```

## Examples

See [`examples/`](examples/) for more:
- `basic_navigation.rs` - Navigation, screenshots, page interactions
- `locator_demo.rs` - Auto-waiting locators and element selection

```bash
cargo run --example basic_navigation
```


## Architecture

Sparkle is built on three main layers:

- **`async_api/`** - High-level API (Browser, Page, Locator)
- **`core/`** - Types, errors, options builders  
- **`driver/`** - WebDriver adapter (wraps thirtyfour)

Key components: `Playwright`, `Browser`, `BrowserContext`, `Page`, `Locator`, `ElementHandle`

## Project Status

**Current:** v0.1 (Early Stage)

**Implemented:**
- Core async API with Chromium support
- Auto-waiting Locators with element interactions
- Screenshot capture & JavaScript evaluation  
- CLI tool with automatic browser/driver management
- Playwright cache compatibility

**In Progress:**
- Network interception
- Frame handling  
- Dialogs & file uploads

**Planned:**
- Firefox & WebKit support
- Video recording
- Complete API parity with Playwright

## Contributing

Contributions welcome! This is an early-stage project with lots of opportunity.

## License

Apache License 2.0 - see [LICENSE](LICENSE)

## Acknowledgments

- [Playwright](https://playwright.dev/) by Microsoft
- [thirtyfour](https://github.com/stevepryde/thirtyfour) - Rust WebDriver client  
- [tokio](https://tokio.rs/) - Async runtime
