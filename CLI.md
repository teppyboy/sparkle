# Sparkle CLI

Command-line tool for managing Chrome and ChromeDriver installations.

## Installation

```bash
# Install from source
cargo install --path .

# Or build locally
cargo build --bin sparkle --release
```

## Commands

### Install

```bash
# Install Chrome + ChromeDriver (default)
sparkle install chrome

# Skip ChromeDriver
sparkle install chrome --skip-driver

# Force reinstall
sparkle install chrome --force
```

**What it does:**
- Downloads latest Playwright Chromium from GitHub
- Installs to Playwright's cache directory (`ms-playwright/`)
- Creates protection markers so Playwright won't remove it
- Automatically downloads matching ChromeDriver

### List

```bash
sparkle list
```

Shows all installed browsers with their revisions and locations.

### Uninstall

```bash
# Uninstall Chrome
sparkle uninstall chrome

# Uninstall ChromeDriver only
sparkle uninstall chromedriver

# Uninstall everything
sparkle uninstall all
```

## Installation Location

**Playwright Cache Directory:**
- Windows: `%LOCALAPPDATA%\ms-playwright`
- Linux: `~/.cache/ms-playwright`
- macOS: `~/Library/Caches/ms-playwright`

Browsers install to `chromium-{revision}/` directories (e.g., `chromium-1208/`).

## Platform Support

Auto-detects and downloads the correct binaries:
- Windows (x64, ARM64)
- Linux (x64, ARM64)
- macOS (Intel, Apple Silicon)

## Using Installed Browsers

### Automatic (Recommended)

Sparkle auto-detects installed browsers and launches ChromeDriver automatically:

```rust
use sparkle::prelude::*;

let playwright = Playwright::new().await?;
let browser = playwright.chromium().launch(Default::default()).await?;
// ChromeDriver launches automatically!
```

### Custom Paths

Override with environment variables:

```bash
# Use custom ChromeDriver
export CHROMEDRIVER_PATH=/path/to/chromedriver

# Connect to existing ChromeDriver server
export CHROMEDRIVER_URL=http://localhost:9515

# Use custom Chrome binary
export CHROME_PATH=/path/to/chrome
```

## Playwright Compatibility

Sparkle shares the browser cache with Playwright:
- Both tools can use the same Chrome installations
- Playwright won't remove Sparkle-installed browsers
- Uses official Playwright revision numbers

To use with Playwright:
```bash
# Install with Sparkle
sparkle install chrome

# Use with Playwright (Node.js)
npx playwright test
# Playwright will detect and use the same Chrome!
```

## Troubleshooting

**Download fails:**
- Check internet connection
- Verify firewall/proxy settings

**Extraction fails:**
- Ensure sufficient disk space
- Check write permissions
- Try with elevated privileges

**ChromeDriver not found:**
- Verify installation: `sparkle list`
- Reinstall: `sparkle install chrome --force`
- Set custom path: `export CHROMEDRIVER_PATH=/path/to/chromedriver`

## Examples

**Fresh setup:**
```bash
sparkle install chrome
cargo run --example basic_navigation
```

**Update to latest:**
```bash
sparkle install chrome --force
```

**Clean uninstall:**
```bash
sparkle uninstall all
```
