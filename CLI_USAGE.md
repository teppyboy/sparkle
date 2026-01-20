# Sparkle CLI Usage Guide

The Sparkle CLI tool helps you manage browser installations for Sparkle automation.

## Installation

```bash
# Install from source
cargo install --path .

# Or build locally
cargo build --bin sparkle --release
./target/release/sparkle --help
```

## Commands

### Install Browsers

Install Chrome and ChromeDriver automatically:

```bash
# Install Chrome (Chromium)
sparkle install chromium

# Install with ChromeDriver
sparkle install chromium --with-deps

# Install everything
sparkle install all

# Force reinstall even if already installed
sparkle install chromium --force
```

### List Installed Browsers

Check which browsers are currently installed:

```bash
sparkle list
```

Output example:
```
Installed Browsers
==================

✓ Chrome
  Location: C:\Users\YourName\AppData\Roaming\sparkle\browsers\chrome

✓ ChromeDriver
  Location: C:\Users\YourName\AppData\Roaming\sparkle\browsers\chromedriver
```

### Uninstall Browsers

Remove installed browsers:

```bash
# Uninstall Chrome
sparkle uninstall chrome

# Uninstall ChromeDriver
sparkle uninstall chromedriver

# Uninstall everything
sparkle uninstall all
```

## Installation Locations

Browsers are installed to platform-specific directories:

- **Windows**: `%APPDATA%\sparkle\browsers`
  - Example: `C:\Users\YourName\AppData\Roaming\sparkle\browsers`
  
- **Linux**: `~/.local/share/sparkle/browsers`
  - Example: `/home/username/.local/share/sparkle/browsers`
  
- **macOS**: `~/Library/Application Support/com.sparkle.browsers`
  - Example: `/Users/username/Library/Application Support/com.sparkle.browsers`

## Platform Support

The CLI automatically detects your platform and downloads the correct binaries:

- ✅ Windows x64
- ✅ Windows ARM64
- ✅ Linux x64
- ✅ Linux ARM64
- ✅ macOS x64 (Intel)
- ✅ macOS ARM64 (Apple Silicon)

## Version Management

The CLI automatically fetches the latest stable Chrome version from Google's Chrome for Testing repository.

## Using Installed Browsers

After installing browsers with the CLI, you can use them in your Sparkle code:

### Option 1: Auto-detection (Coming Soon)

Sparkle will automatically detect CLI-installed browsers.

### Option 2: Manual Path Configuration

Set environment variables to point to the installed browsers:

```bash
# Windows (PowerShell)
$env:CHROME_PATH="C:\Users\YourName\AppData\Roaming\sparkle\browsers\chrome"
$env:CHROMEDRIVER_PATH="C:\Users\YourName\AppData\Roaming\sparkle\browsers\chromedriver"

# Linux/macOS (Bash/Zsh)
export CHROME_PATH="$HOME/.local/share/sparkle/browsers/chrome"
export CHROMEDRIVER_PATH="$HOME/.local/share/sparkle/browsers/chromedriver"
```

### Option 3: Programmatic Configuration

```rust
use sparkle::prelude::*;

let options = LaunchOptionsBuilder::default()
    .executable_path("/path/to/chrome/binary")
    .build()?;

let browser = playwright.chromium().launch(options).await?;
```

## Troubleshooting

### Download fails

If downloads fail, check:
1. Internet connection
2. Firewall/proxy settings
3. Try again with a different network

### Extraction fails

If extraction fails:
1. Check disk space
2. Ensure you have write permissions to the installation directory
3. Try running with administrator/sudo privileges

### Binary not executable (Linux/macOS)

The CLI automatically sets executable permissions. If you still have issues:

```bash
chmod +x ~/.local/share/sparkle/browsers/chrome/chrome
chmod +x ~/.local/share/sparkle/browsers/chromedriver/chromedriver
```

## Examples

### Fresh Installation

```bash
# Install Sparkle CLI
cargo install --path .

# Install browsers
sparkle install all

# Verify installation
sparkle list

# Use in your project
cargo run --example basic_navigation
```

### Update to Latest Version

```bash
# Reinstall to get latest version
sparkle install chromium --force
```

### Clean Uninstall

```bash
# Remove all installed browsers
sparkle uninstall all

# Verify removal
sparkle list
```

## Advanced Usage

### Custom Installation Directory

Currently, the installation directory is determined by the system. To use a custom location, you can:

1. Install using the CLI to the default location
2. Copy/move the browsers to your desired location
3. Update your code to point to the new location

### Offline Installation

The CLI requires internet access to download browsers. For offline installation:

1. Download Chrome and ChromeDriver manually from https://googlechromelabs.github.io/chrome-for-testing/
2. Extract to the standard installation directory
3. Use `sparkle list` to verify they're detected

## Contributing

Found a bug or want to improve the CLI? Contributions welcome!

See the main README.md for contribution guidelines.
