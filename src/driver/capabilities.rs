//! Browser capabilities and configuration
//!
//! This module provides utilities for constructing browser capabilities
//! for different browser types.

use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::core::StealthOptions;

/// Builder for Chromium capabilities
pub struct ChromiumCapabilities {
    headless: bool,
    args: Vec<String>,
    binary: Option<PathBuf>,
    env: HashMap<String, String>,
    prefs: HashMap<String, serde_json::Value>,
    downloads_path: Option<PathBuf>,
    stealth: Option<StealthOptions>,
}

impl ChromiumCapabilities {
    /// Create a new capabilities builder
    pub fn new() -> Self {
        Self {
            headless: true,
            args: Vec::new(),
            binary: None,
            env: HashMap::new(),
            prefs: HashMap::new(),
            downloads_path: None,
            stealth: None,
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

    /// Add an environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Add multiple environment variables
    pub fn envs(mut self, vars: HashMap<String, String>) -> Self {
        self.env.extend(vars);
        self
    }

    /// Get the environment variables
    pub fn get_env(&self) -> &HashMap<String, String> {
        &self.env
    }

    /// Set a Chrome preference
    pub fn pref(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.prefs.insert(key.into(), value);
        self
    }

    /// Add multiple Chrome preferences
    pub fn prefs(mut self, prefs: HashMap<String, serde_json::Value>) -> Self {
        self.prefs.extend(prefs);
        self
    }

    /// Set the downloads directory path
    pub fn downloads_path(mut self, path: PathBuf) -> Self {
        self.downloads_path = Some(path);
        self
    }

    /// Set stealth options
    pub fn stealth(mut self, stealth: StealthOptions) -> Self {
        self.stealth = Some(stealth);
        self
    }

    /// Add proxy configuration via command-line arguments
    pub fn proxy(mut self, server: &str, bypass: Option<&str>) -> Self {
        self.args.push(format!("--proxy-server={}", server));
        if let Some(bypass_list) = bypass {
            self.args
                .push(format!("--proxy-bypass-list={}", bypass_list));
        }
        self
    }

    /// Build the capabilities as a HashMap
    pub fn build(self) -> HashMap<String, serde_json::Value> {
        let mut args = self.args;

        // Apply stealth mode switches (Patchright-style)
        let stealth_enabled = self.stealth.as_ref().map(|s| s.enabled).unwrap_or(true);

        if stealth_enabled {
            // Add stealth-specific arguments
            args.push("--disable-blink-features=AutomationControlled".to_string());

            // Remove automation-revealing arguments that might be added elsewhere
            // Patchright removes these to avoid detection:
            args.retain(|arg| {
                !arg.contains("--enable-automation")
                    && !arg.contains("--disable-popup-blocking")
                    && !arg.contains("--disable-component-update")
                    && !arg.contains("--disable-default-apps")
                    && !arg.contains("--disable-extensions")
                    && !arg.contains("--disable-client-side-phishing-detection")
                    && !arg.contains("--disable-component-extensions-with-background-pages")
                    && !arg.contains("--allow-pre-commit-input")
                    && !arg.contains("--disable-ipc-flooding-protection")
                    && !arg.contains("--metrics-recording-only")
                    && !arg.contains("--unsafely-disable-devtools-self-xss-warnings")
                    && !arg.contains("--disable-back-forward-cache")
            });
        }

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

        // Add environment variables if specified
        if !self.env.is_empty() {
            if let Some(obj) = chrome_options.as_object_mut() {
                // Chrome uses "env" in experimental options for browser process env vars
                let mut experimental = serde_json::Map::new();
                experimental.insert("env".to_string(), json!(self.env));
                obj.insert("experimentalOptions".to_string(), json!(experimental));
            }
        }

        // Add preferences (including downloads_path) if specified
        let mut all_prefs = self.prefs;
        if let Some(downloads_dir) = self.downloads_path {
            all_prefs.insert(
                "download.default_directory".to_string(),
                json!(downloads_dir.to_string_lossy()),
            );
            all_prefs.insert("download.prompt_for_download".to_string(), json!(false));
        }

        if !all_prefs.is_empty() {
            if let Some(obj) = chrome_options.as_object_mut() {
                obj.insert("prefs".to_string(), json!(all_prefs));
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
