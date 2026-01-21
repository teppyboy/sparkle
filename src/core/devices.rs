//! Device descriptors for mobile and tablet emulation
//!
//! This module provides a curated list of device descriptors from Playwright,
//! allowing easy emulation of various mobile devices and tablets.
//!
//! Device descriptors are fetched dynamically from the official Playwright repository
//! on first access, ensuring you always have the latest device definitions.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::error::{Error, Result};
use super::options::{BrowserContextOptions, BrowserContextOptionsBuilder, ViewportSize};

/// URL to Playwright's device descriptors JSON
const PLAYWRIGHT_DEVICES_URL: &str = "https://raw.githubusercontent.com/microsoft/playwright/main/packages/playwright-core/src/server/deviceDescriptorsSource.json";

/// A device descriptor with viewport, user agent, and other properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDescriptor {
    /// User agent string for this device
    #[serde(rename = "userAgent")]
    pub user_agent: String,

    /// Viewport size (logical pixels)
    pub viewport: ViewportSize,

    /// Device scale factor (device pixel ratio)
    #[serde(rename = "deviceScaleFactor")]
    pub device_scale_factor: f64,

    /// Whether the device is a mobile device
    #[serde(rename = "isMobile")]
    pub is_mobile: bool,

    /// Whether the device supports touch events
    #[serde(rename = "hasTouch")]
    pub has_touch: bool,

    /// Default browser type for this device (chromium, firefox, webkit)
    #[serde(rename = "defaultBrowserType")]
    pub default_browser_type: String,

    /// Screen size (physical pixels) - optional, some devices don't specify
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen: Option<ViewportSize>,
}

impl DeviceDescriptor {
    /// Convert this device descriptor to BrowserContextOptions
    ///
    /// This creates a `BrowserContextOptions` instance pre-configured with
    /// the device's viewport, user agent, device scale factor, mobile flag,
    /// and touch support.
    ///
    /// # Example
    /// ```no_run
    /// # use sparkle::core::devices::get_device;
    /// # async fn example() -> sparkle::core::Result<()> {
    /// let iphone = get_device("iPhone 12").await?;
    /// let options = iphone.to_context_options();
    /// // Use options with browser.new_context()
    /// # Ok(())
    /// # }
    /// ```
    pub fn to_context_options(&self) -> BrowserContextOptions {
        BrowserContextOptionsBuilder::default()
            .user_agent(self.user_agent.clone())
            .viewport(self.viewport)
            .device_scale_factor(self.device_scale_factor)
            .is_mobile(self.is_mobile)
            .has_touch(self.has_touch)
            .build()
            .unwrap()
    }
}

/// Device registry that lazily fetches devices from Playwright
struct DeviceRegistry {
    devices: RwLock<Option<HashMap<String, DeviceDescriptor>>>,
    fetch_lock: tokio::sync::Mutex<()>,
}

impl DeviceRegistry {
    fn new() -> Self {
        Self {
            devices: RwLock::new(None),
            fetch_lock: tokio::sync::Mutex::new(()),
        }
    }

    /// Fetch devices from Playwright repository
    /// This method is thread-safe and ensures only one fetch happens even with concurrent calls
    async fn fetch_devices(&self) -> Result<()> {
        // Quick check without locking if already loaded
        {
            let devices = self.devices.read().await;
            if devices.is_some() {
                return Ok(());
            }
        }

        // Acquire fetch lock to prevent concurrent fetches
        let _fetch_guard = self.fetch_lock.lock().await;

        // Double-check after acquiring lock (another thread might have fetched)
        {
            let devices = self.devices.read().await;
            if devices.is_some() {
                return Ok(());
            }
        }

        // Fetch from Playwright
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| Error::ActionFailed(format!("Failed to create HTTP client: {}", e)))?;

        let response = client
            .get(PLAYWRIGHT_DEVICES_URL)
            .send()
            .await
            .map_err(|e| {
                Error::ActionFailed(format!("Failed to fetch device descriptors: {}", e))
            })?;

        let json_text = response.text().await.map_err(|e| {
            Error::ActionFailed(format!("Failed to read device descriptors response: {}", e))
        })?;

        let parsed: HashMap<String, DeviceDescriptor> =
            serde_json::from_str(&json_text).map_err(|e| {
                Error::ActionFailed(format!("Failed to parse device descriptors JSON: {}", e))
            })?;

        // Store the loaded devices
        let mut devices = self.devices.write().await;
        *devices = Some(parsed);

        Ok(())
    }

    /// Get a device by name
    async fn get(&self, name: &str) -> Result<Option<DeviceDescriptor>> {
        self.fetch_devices().await?;
        let devices = self.devices.read().await;
        Ok(devices.as_ref().and_then(|d| d.get(name).cloned()))
    }

    /// List all device names
    async fn list(&self) -> Result<Vec<String>> {
        self.fetch_devices().await?;
        let devices = self.devices.read().await;
        if let Some(devices) = devices.as_ref() {
            let mut names: Vec<String> = devices.keys().cloned().collect();
            names.sort();
            Ok(names)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get all devices
    async fn all(&self) -> Result<HashMap<String, DeviceDescriptor>> {
        self.fetch_devices().await?;
        let devices = self.devices.read().await;
        Ok(devices.as_ref().cloned().unwrap_or_default())
    }
}

/// Global device registry
static DEVICE_REGISTRY: Lazy<Arc<DeviceRegistry>> = Lazy::new(|| Arc::new(DeviceRegistry::new()));

/// Get a device descriptor by name
///
/// Returns `None` if the device name is not found.
/// Automatically fetches device list from Playwright on first call.
///
/// # Example
/// ```no_run
/// # use sparkle::core::devices::get_device;
/// # async fn example() -> sparkle::core::Result<()> {
/// let iphone = get_device("iPhone 12").await?;
/// if let Some(device) = iphone {
///     println!("User agent: {}", device.user_agent);
///     println!("Viewport: {}x{}", device.viewport.width, device.viewport.height);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn get_device(name: &str) -> Result<Option<DeviceDescriptor>> {
    DEVICE_REGISTRY.get(name).await
}

/// List all available device names
///
/// Automatically fetches device list from Playwright on first call.
///
/// # Example
/// ```no_run
/// # use sparkle::core::devices::list_devices;
/// # async fn example() -> sparkle::core::Result<()> {
/// for device_name in list_devices().await? {
///     println!("{}", device_name);
/// }
/// # Ok(())
/// # }
/// ```
pub async fn list_devices() -> Result<Vec<String>> {
    DEVICE_REGISTRY.list().await
}

/// Get all device descriptors
///
/// Returns a HashMap of all available device descriptors.
/// Automatically fetches device list from Playwright on first call.
///
/// # Example
/// ```no_run
/// # use sparkle::core::devices::get_all_devices;
/// # async fn example() -> sparkle::core::Result<()> {
/// let devices = get_all_devices().await?;
/// println!("Found {} devices", devices.len());
/// # Ok(())
/// # }
/// ```
pub async fn get_all_devices() -> Result<HashMap<String, DeviceDescriptor>> {
    DEVICE_REGISTRY.all().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_devices_loaded() {
        let devices = get_all_devices().await.unwrap();
        assert!(!devices.is_empty(), "Device descriptors should be loaded");
        assert!(
            devices.len() > 100,
            "Should have at least 100 device descriptors"
        );
    }

    #[tokio::test]
    async fn test_iphone_12() {
        let iphone = get_device("iPhone 12")
            .await
            .unwrap()
            .expect("iPhone 12 should exist");
        assert_eq!(iphone.viewport.width, 390);
        assert_eq!(iphone.viewport.height, 664);
        assert_eq!(iphone.device_scale_factor, 3.0);
        assert!(iphone.is_mobile);
        assert!(iphone.has_touch);
        assert!(iphone.user_agent.contains("iPhone"));
        assert_eq!(iphone.default_browser_type, "webkit");
    }

    #[tokio::test]
    async fn test_pixel_5() {
        let pixel = get_device("Pixel 5")
            .await
            .unwrap()
            .expect("Pixel 5 should exist");
        assert_eq!(pixel.viewport.width, 393);
        assert_eq!(pixel.viewport.height, 727);
        assert_eq!(pixel.device_scale_factor, 2.75);
        assert!(pixel.is_mobile);
        assert!(pixel.has_touch);
        assert!(pixel.user_agent.contains("Pixel 5"));
        assert_eq!(pixel.default_browser_type, "chromium");
    }

    #[tokio::test]
    async fn test_ipad_pro_11() {
        let ipad = get_device("iPad Pro 11")
            .await
            .unwrap()
            .expect("iPad Pro 11 should exist");
        assert_eq!(ipad.viewport.width, 834);
        assert_eq!(ipad.viewport.height, 1194);
        assert_eq!(ipad.device_scale_factor, 2.0);
        assert!(ipad.is_mobile);
        assert!(ipad.has_touch);
    }

    #[tokio::test]
    async fn test_to_context_options() {
        let iphone = get_device("iPhone 12").await.unwrap().unwrap();
        let options = iphone.to_context_options();

        assert_eq!(options.user_agent, Some(iphone.user_agent.clone()));
        assert_eq!(options.viewport, Some(iphone.viewport));
        assert_eq!(
            options.device_scale_factor,
            Some(iphone.device_scale_factor)
        );
        assert_eq!(options.is_mobile, Some(iphone.is_mobile));
        assert_eq!(options.has_touch, Some(iphone.has_touch));
    }

    #[tokio::test]
    async fn test_list_devices() {
        let devices = list_devices().await.unwrap();
        assert!(!devices.is_empty());
        assert!(devices.contains(&"iPhone 12".to_string()));
        assert!(devices.contains(&"Pixel 5".to_string()));

        // Verify sorted
        let mut sorted = devices.clone();
        sorted.sort();
        assert_eq!(devices, sorted);
    }

    #[tokio::test]
    async fn test_nonexistent_device() {
        let result = get_device("Nonexistent Device 9000").await.unwrap();
        assert!(result.is_none());
    }
}
