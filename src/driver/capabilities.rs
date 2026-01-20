//! Browser capabilities and configuration
//!
//! This module provides utilities for constructing browser capabilities
//! for different browser types.

use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

/// Builder for Chromium capabilities
pub struct ChromiumCapabilities {
    headless: bool,
    args: Vec<String>,
    binary: Option<PathBuf>,
}

impl ChromiumCapabilities {
    /// Create a new capabilities builder
    pub fn new() -> Self {
        Self {
            headless: true,
            args: Vec::new(),
            binary: None,
        }
    }

    /// Set headless mode
    pub fn headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }

    /// Add a command-line argument
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple command-line arguments
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(|a| a.into()));
        self
    }

    /// Set the Chrome binary path
    pub fn binary(mut self, binary: PathBuf) -> Self {
        self.binary = Some(binary);
        self
    }

    /// Build the capabilities as a HashMap
    pub fn build(self) -> HashMap<String, serde_json::Value> {
        let mut args = self.args;

        if self.headless {
            args.push("--headless".to_string());
            args.push("--disable-gpu".to_string());
        }

        let mut chrome_options = json!({
            "args": args,
        });

        // Add binary path if specified
        if let Some(binary) = self.binary {
            if let Some(obj) = chrome_options.as_object_mut() {
                obj.insert("binary".to_string(), json!(binary.to_string_lossy()));
            }
        }

        let mut caps = HashMap::new();
        caps.insert("browserName".to_string(), json!("chrome"));
        caps.insert("goog:chromeOptions".to_string(), chrome_options);

        caps
    }
}

impl Default for ChromiumCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chromium_capabilities() {
        let caps = ChromiumCapabilities::new()
            .headless(true)
            .arg("--disable-dev-shm-usage")
            .build();

        assert_eq!(caps.get("browserName").unwrap(), &json!("chrome"));
        assert!(caps.contains_key("goog:chromeOptions"));
    }

    #[test]
    fn test_chromium_capabilities_with_args() {
        let caps = ChromiumCapabilities::new()
            .headless(false)
            .args(vec!["--window-size=1920,1080", "--start-maximized"])
            .build();

        assert!(caps.contains_key("goog:chromeOptions"));
    }
}
