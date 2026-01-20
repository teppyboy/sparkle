# Agent Development Guide

This document provides coding guidelines and conventions for AI agents and developers working on the Sparkle browser automation library.

## Project Overview

**Sparkle** is a Rust reimplementation of **Playwright's official Python API**, powered by thirtyfour (WebDriver) and Tokio for async operations.

### Goals
- **API Compatibility**: Maintain API compatibility with Playwright Python to ease migration and leverage existing knowledge
- **Async Runtime**: Built on Tokio for high-performance async I/O operations
- **Type Safety**: Leverage Rust's type system for compile-time safety while keeping the API ergonomic
- **WebDriver Backend**: Uses thirtyfour to communicate with WebDriver (ChromeDriver, GeckoDriver, etc.)

### Architecture
```
┌─────────────────────────────────────┐
│   Playwright Python API (Target)   │
└──────────────┬──────────────────────┘
               │ API-compatible
┌──────────────▼──────────────────────┐
│   Sparkle (Rust async_api)          │ ← You are here
│   - Browser, Page, Locator          │
│   - Built with Tokio async/await    │
└──────────────┬──────────────────────┘
               │ Adapter layer
┌──────────────▼──────────────────────┐
│   WebDriver Adapter (driver/)       │
│   - Wraps thirtyfour                │
└──────────────┬──────────────────────┘
               │ WebDriver protocol
┌──────────────▼──────────────────────┐
│   ChromeDriver / WebDriver          │
└─────────────────────────────────────┘
```

### Design Principles
1. **Match Playwright Python's API** - Method names, parameters, and behavior should mirror the Python implementation
2. **Async by default** - All I/O operations use Tokio async/await (not blocking)
3. **Builder pattern for options** - Complex options use derive_builder for ergonomic configuration
4. **Explicit error handling** - Use Result<T> and custom Error types, never panic in library code
5. **Auto-waiting and retry** - Locators automatically wait and retry like Playwright (not fully implemented yet)

## Build, Lint, and Test Commands

### Building
```bash
# Build library and binary
cargo build

# Build release version
cargo build --release

# Build examples
cargo build --examples

# Build specific example
cargo build --example basic_navigation
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in specific module
cargo test module_name::

# Run async tests (using tokio-test)
cargo test --features tokio-test

# Run examples (integration testing)
cargo run --example basic_navigation
cargo run --example wikipedia_search
cargo run --example locator_demo
```

### Linting
```bash
# Run Clippy (default lints)
cargo clippy

# Clippy with all warnings as errors
cargo clippy -- -D warnings

# Format code (uses default rustfmt)
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### CLI Testing
```bash
# Install Chrome with ChromeDriver
./target/release/sparkle.exe install chrome

# List installed browsers
./target/release/sparkle.exe list

# Uninstall browser
./target/release/sparkle.exe uninstall chrome
```

## Code Style Guidelines

### Import Organization

Always organize imports in three tiers with blank lines between:

```rust
// 1. Standard library imports
use std::sync::Arc;
use std::time::Duration;

// 2. External crate imports
use thirtyfour::prelude::*;
use tokio::sync::RwLock;
use derive_builder::Builder;

// 3. Internal module imports (using crate::)
use crate::core::{Error, Result};
use crate::driver::WebDriverAdapter;
```

### Documentation

**Module-level documentation** (at file start):
```rust
//! Module name and purpose
//!
//! Detailed description of what this module provides.
```

**Item-level documentation** (before functions, structs, etc.):
```rust
/// Brief one-line summary
///
/// Detailed description when needed.
///
/// # Arguments
/// * `param` - Description of parameter
///
/// # Returns
/// Description of return value
///
/// # Example
/// ```no_run
/// # use sparkle::async_api::Page;
/// # async fn example(page: &Page) -> sparkle::core::Result<()> {
/// page.goto("https://example.com", Default::default()).await?;
/// # Ok(())
/// # }
/// ```
pub async fn goto(&self, url: &str) -> Result<()> { ... }
```

**Documentation conventions:**
- Use `# ` prefix for hidden example setup lines
- Use `no_run` attribute for examples that require external resources
- Include examples for all public API methods
- Document error conditions when relevant

### Error Handling

**Always use the custom Result type:**
```rust
use crate::core::{Error, Result};

pub async fn method(&self) -> Result<ReturnType> {
    // Use ? operator for error propagation
    let value = self.some_operation().await?;
    Ok(value)
}
```

**Create domain-specific errors:**
```rust
// Use error helper methods
return Err(Error::element_not_found(selector));
return Err(Error::timeout_duration("operation", timeout));
return Err(Error::not_implemented("feature"));
```

**Convert external errors:**
```rust
// Automatic conversion via #[from] attribute
external_call().await?;

// Manual conversion
external_call().await.map_err(|e| {
    Error::ActionFailed(format!("Failed to click: {}", e))
})?;
```

### Naming Conventions

- **Structs/Enums/Traits**: `PascalCase` (e.g., `Browser`, `LaunchOptions`)
- **Functions/methods**: `snake_case` (e.g., `new_page()`, `goto()`)
- **Variables**: `snake_case` (e.g., `browser_version`, `search_input`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_TIMEOUT`)
- **Type aliases**: `PascalCase` (e.g., `type Result<T> = ...`)
- **Rust keywords as identifiers**: Use `r#` prefix (e.g., `r#type()`)

### Async Patterns

**All public API methods are async:**
```rust
pub async fn method_name(&self, param: Type) -> Result<ReturnType> {
    let result = async_operation().await?;
    Ok(result)
}
```

**Shared mutable state pattern:**
```rust
pub struct Browser {
    adapter: Arc<WebDriverAdapter>,
    closed: Arc<RwLock<bool>>,
}

pub async fn close(&self) -> Result<()> {
    let mut closed = self.closed.write().await;
    if !*closed {
        *closed = true;
        // cleanup
    }
    Ok(())
}
```

**Timeout and retry pattern:**
```rust
let start = std::time::Instant::now();
loop {
    match self.try_operation().await {
        Ok(result) => return Ok(result),
        Err(_) if start.elapsed() >= timeout => break,
        Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
    }
}
Err(Error::timeout_duration("operation", timeout))
```

### Builder Pattern

**Use derive_builder for options:**
```rust
#[derive(Debug, Clone, Builder, Default)]
#[builder(default, setter(into, strip_option))]
pub struct LaunchOptions {
    pub headless: Option<bool>,
    pub timeout: Option<Duration>,
}

// Usage:
let options = LaunchOptionsBuilder::default()
    .headless(true)
    .timeout(Duration::from_secs(30))
    .build()
    .unwrap();
```

**Manual fluent builders for complex types:**
```rust
impl ChromiumCapabilities {
    pub fn new() -> Self { ... }
    
    pub fn headless(mut self, value: bool) -> Self {
        self.headless = value;
        self
    }
    
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }
}
```

### Type Safety

- **Avoid `.unwrap()`** in library code; use `?` operator instead
- **Use `impl Into<String>`** for string parameters to accept `&str` or `String`
- **Prefer `&str`** for temporary string usage
- **Use `Option<T>`** for optional fields in option structs
- **Use `Arc<T>`** for shared ownership in async contexts

## Testing Guidelines

**Write inline unit tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_function() {
        let result = sync_function();
        assert_eq!(result, expected);
    }

    #[tokio::test]
    async fn test_async_function() {
        let result = async_function().await.unwrap();
        assert!(result.is_valid());
    }
}
```

**Testing location:**
- Inline tests at bottom of source files using `#[cfg(test)]` modules
- Integration tests can go in `tests/` directory (currently unused)
- Examples in `examples/` serve as integration tests

## Module Organization

- **`src/async_api/`** - High-level Playwright-like API (Browser, Page, Locator)
- **`src/driver/`** - WebDriver adapter layer (wraps thirtyfour)
- **`src/core/`** - Core types (Error, Result, Options)
- **`src/cli/`** - CLI commands (install, list, uninstall)
- **`src/bin/`** - Binary entry points

**Module exports:**
```rust
// In mod.rs:
pub mod submodule;
pub use submodule::PublicType;

// In lib.rs prelude:
pub mod prelude {
    pub use crate::async_api::*;
    pub use crate::core::*;
}
```

## Common Patterns

### Visibility
- `pub` - Public API
- `pub(crate)` - Internal to library
- No modifier - Private to module

### Internal constructors
```rust
pub struct Browser {
    adapter: Arc<WebDriverAdapter>,
}

impl Browser {
    // Internal constructor
    pub(crate) fn new(adapter: Arc<WebDriverAdapter>) -> Self {
        Self { adapter }
    }
}
```

### Chrome DevTools Protocol (CDP)
```rust
use thirtyfour::extensions::cdp::ChromeDevTools;

let dev_tools = ChromeDevTools::new(driver.handle.clone());
let result = dev_tools.execute_cdp("Browser.getVersion").await?;
```

## Special Notes

- **No emojis** in code or documentation unless explicitly requested
- **Clean output** - Use `println!` sparingly, prefer tracing for debug info
- **Error messages** should be descriptive and actionable
- **CLI uses `anyhow::Result`**, library uses custom `Result<T>`
- **ChromeDriver auto-launches** - no manual process management needed
- **Playwright compatibility** - API design matches Python Playwright where possible
- **Tokio-based async** - All async operations use Tokio runtime, not other async runtimes

## API Design Philosophy

When implementing new features:

1. **Check Playwright Python documentation first** - The API should match Playwright Python's behavior
2. **Method signatures** - Match Python parameter names and types (translated to Rust idioms)
3. **Return types** - Use `Result<T>` for operations that can fail, matching Playwright's error handling
4. **Options pattern** - Use builder pattern for optional parameters (e.g., `LaunchOptions`, `NavigationOptions`)
5. **Auto-waiting** - Locators should wait for elements to be ready before acting (like Playwright)
6. **Async all the way** - All I/O operations must be async using Tokio's `async/await`

### Example API Mapping

**Playwright Python:**
```python
page.goto("https://example.com", wait_until="networkidle")
page.locator("button").click()
page.fill("#username", "test")
```

**Sparkle Rust:**
```rust
page.goto("https://example.com", NavigationOptionsBuilder::default()
    .wait_until(WaitUntilState::NetworkIdle)
    .build()
    .unwrap()
).await?;
page.locator("button").click(Default::default()).await?;
page.locator("#username").fill("test").await?;
```

## Running the Project

```bash
# Build and run CLI
cargo run --release -- install chrome
cargo run --release -- list

# Run examples
cargo run --example basic_navigation --release
cargo run --example wikipedia_search --release

# Test library functionality
cargo test
```
