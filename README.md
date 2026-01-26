# Sparkle

> [!WARNING]
> This project is mostly "vibe-coded", **many** features are not properly implemented (by using hacky workaround, etc.), unknown bugs will appear at any time. While I try to fix it (either by vibing or actually coding), not all bugs are guaranteed to be fixed.

> A reimplementation of Playwright written in Rust, powered by `thirtyfour`.

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Sparkle brings Playwright's API to Rust with async support.

## Features

- **Playwright-like API** - Easy migration from Python to Rust
- **CLI Tool** - Automatic browser/driver download and management
- **Playwright Compatible** - Shares browser cache with Playwright installations
- **Zero Config** - ChromeDriver auto-launches and manages itself
- **Fast & Ergonomic** - Idiomatic Rust with minimal boilerplate

### Implemented features

These are the features that I've tested to see whether it works or not.

| Feature | Working |
|---------|---------|
|Locator|    ✅    |
|Locator (iframe)|   ❌      |
|         |         |

## Documentation

Documentation is available at https://sparkle.tretrauit.me, ~~and maybe https://docs.rs if I publish this crate later.~~

## Installation

### 1. Install Sparkle CLI

```bash
cargo install --git https://github.com/teppyboy/sparkle
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

## CLI

See [CLI.md](./CLI.md) for more information.

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

### Logging

Sparkle uses [tracing](https://github.com/tokio-rs/tracing) for structured logging. Enable logs by setting the `SPARKLE_LOG_LEVEL` environment variable:

```bash
# Set log level (trace, debug, info, warn, error)
export SPARKLE_LOG_LEVEL=info
```

**In your code:**

```rust
use sparkle::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging from SPARKLE_LOG_LEVEL environment variable
    init_logging();
    
    // Your automation code here...
    Ok(())
}
```

## Examples

See [`examples/`](examples/) for how to use the library, to run an example:

```bash
cargo run --example basic_navigation
```


## Architecture

Sparkle is built on three main layers:

- **`async_api/`** - High-level API (Browser, Page, Locator)
- **`core/`** - Types, errors, options builders  
- **`driver/`** - WebDriver adapter (wraps thirtyfour)

## Developing

This project uses OpenSkills, so before working on the project, install the needed skills:

```bash
bunx --bun openskills install ZhangHanDong/rust-skills
bunx --bun openskills install lackeyjb/playwright-skill 
```

## Project Status

This project is in very early stage, do NOT expect anything to work yet.

## License

[Apache License 2.0](./LICENSE)

## Acknowledgments

- [Playwright](https://playwright.dev/) by Microsoft
- [thirtyfour](https://github.com/stevepryde/thirtyfour) - Rust WebDriver client  
- [tokio](https://tokio.rs/) - Async runtime
