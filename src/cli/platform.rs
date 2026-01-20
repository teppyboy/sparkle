//! Platform detection and configuration
//!
//! Detects the current operating system and architecture to determine
//! which browser binaries to download.

use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    WindowsX64,
    WindowsArm64,
    LinuxX64,
    LinuxArm64,
    MacOsX64,
    MacOsArm64,
}

impl Platform {
    /// Detect the current platform
    pub fn detect() -> anyhow::Result<Self> {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;

        match (os, arch) {
            ("windows", "x86_64") => Ok(Platform::WindowsX64),
            ("windows", "aarch64") => Ok(Platform::WindowsArm64),
            ("linux", "x86_64") => Ok(Platform::LinuxX64),
            ("linux", "aarch64") => Ok(Platform::LinuxArm64),
            ("macos", "x86_64") => Ok(Platform::MacOsX64),
            ("macos", "aarch64") => Ok(Platform::MacOsArm64),
            _ => Err(anyhow::anyhow!("Unsupported platform: {} {}", os, arch)),
        }
    }

    /// Get the Chrome download URL for this platform
    pub fn chrome_download_url(&self, version: &str) -> String {
        match self {
            Platform::WindowsX64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/win64/chrome-win64.zip",
                version
            ),
            Platform::WindowsArm64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/win-arm64/chrome-win-arm64.zip",
                version
            ),
            Platform::LinuxX64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/linux64/chrome-linux64.zip",
                version
            ),
            Platform::LinuxArm64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/linux-arm64/chrome-linux-arm64.zip",
                version
            ),
            Platform::MacOsX64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/mac-x64/chrome-mac-x64.zip",
                version
            ),
            Platform::MacOsArm64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/mac-arm64/chrome-mac-arm64.zip",
                version
            ),
        }
    }

    /// Get the ChromeDriver download URL for this platform
    pub fn chromedriver_download_url(&self, version: &str) -> String {
        match self {
            Platform::WindowsX64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/win64/chromedriver-win64.zip",
                version
            ),
            Platform::WindowsArm64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/win-arm64/chromedriver-win-arm64.zip",
                version
            ),
            Platform::LinuxX64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/linux64/chromedriver-linux64.zip",
                version
            ),
            Platform::LinuxArm64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/linux-arm64/chromedriver-linux-arm64.zip",
                version
            ),
            Platform::MacOsX64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/mac-x64/chromedriver-mac-x64.zip",
                version
            ),
            Platform::MacOsArm64 => format!(
                "https://storage.googleapis.com/chrome-for-testing-public/{}/mac-arm64/chromedriver-mac-arm64.zip",
                version
            ),
        }
    }

    /// Get the executable file extension for this platform
    pub fn executable_extension(&self) -> &str {
        match self {
            Platform::WindowsX64 | Platform::WindowsArm64 => ".exe",
            _ => "",
        }
    }

    /// Get the platform name as a string
    pub fn name(&self) -> &str {
        match self {
            Platform::WindowsX64 => "windows-x64",
            Platform::WindowsArm64 => "windows-arm64",
            Platform::LinuxX64 => "linux-x64",
            Platform::LinuxArm64 => "linux-arm64",
            Platform::MacOsX64 => "macos-x64",
            Platform::MacOsArm64 => "macos-arm64",
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        assert!(platform.is_ok());
    }

    #[test]
    fn test_chrome_url() {
        let platform = Platform::WindowsX64;
        let url = platform.chrome_download_url("120.0.6099.109");
        assert!(url.contains("win64"));
        assert!(url.contains("chrome"));
    }

    #[test]
    fn test_chromedriver_url() {
        let platform = Platform::LinuxX64;
        let url = platform.chromedriver_download_url("120.0.6099.109");
        assert!(url.contains("linux64"));
        assert!(url.contains("chromedriver"));
    }
}
