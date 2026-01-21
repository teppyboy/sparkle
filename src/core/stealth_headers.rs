//! Header generation for stealth mode
//!
//! This module generates realistic browser headers that match the Chrome version
//! and avoid detection by anti-bot systems.

use crate::core::StealthOptions;

/// Generate User-Agent string for Chrome
///
/// # Arguments
/// * `chrome_version` - Chrome version (e.g., "120.0.6099.129")
/// * `platform` - Platform string (e.g., "Win32", "MacIntel", "Linux x86_64")
pub fn generate_user_agent(chrome_version: &str, platform: &str) -> String {
    match platform {
        "Win32" => {
            format!(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                chrome_version
            )
        }
        "MacIntel" => {
            format!(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                chrome_version
            )
        }
        "Linux x86_64" => {
            format!(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                chrome_version
            )
        }
        _ => {
            // Default to Windows
            format!(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                chrome_version
            )
        }
    }
}

/// Generate sec-ch-ua header for Chrome
///
/// # Arguments
/// * `chrome_version` - Chrome version (e.g., "120.0.6099.129")
pub fn generate_sec_ch_ua(chrome_version: &str) -> String {
    let major_version = chrome_version.split('.').next().unwrap_or("120");

    format!(
        r#""Not_A Brand";v="8", "Chromium";v="{}", "Google Chrome";v="{}""#,
        major_version, major_version
    )
}

/// Generate sec-ch-ua-mobile header (always "?0" for desktop)
pub fn generate_sec_ch_ua_mobile() -> String {
    "?0".to_string()
}

/// Generate sec-ch-ua-platform header
///
/// # Arguments
/// * `platform` - Platform string (e.g., "Win32", "MacIntel", "Linux x86_64")
pub fn generate_sec_ch_ua_platform(platform: &str) -> String {
    match platform {
        "Win32" => r#""Windows""#.to_string(),
        "MacIntel" => r#""macOS""#.to_string(),
        "Linux x86_64" => r#""Linux""#.to_string(),
        _ => r#""Windows""#.to_string(),
    }
}

/// Generate Accept-Language header from locale
///
/// # Arguments
/// * `locale` - Locale string (e.g., "en-US", "en-GB", "fr-FR")
pub fn generate_accept_language(locale: &str) -> String {
    // Split locale into language and region
    if let Some((lang, _region)) = locale.split_once('-') {
        // Format: "en-US,en;q=0.9"
        format!("{},{};q=0.9", locale, lang)
    } else {
        // Just language code
        format!("{};q=0.9", locale)
    }
}

/// Get platform string from OS
pub fn get_platform_string() -> String {
    #[cfg(target_os = "windows")]
    return "Win32".to_string();

    #[cfg(target_os = "macos")]
    return "MacIntel".to_string();

    #[cfg(target_os = "linux")]
    return "Linux x86_64".to_string();

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "Win32".to_string();
}

/// Headers configuration for CDP Network.setUserAgentOverride
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadersConfig {
    pub user_agent: String,
    pub accept_language: String,
    pub platform: String,
}

impl HeadersConfig {
    /// Create headers config from stealth options and browser version
    pub fn from_stealth_options(stealth_options: &StealthOptions, chrome_version: &str) -> Self {
        let platform = get_platform_string();
        let user_agent = generate_user_agent(chrome_version, &platform);
        let locale = stealth_options.locale.as_deref().unwrap_or("en-US");
        let accept_language = generate_accept_language(locale);

        Self {
            user_agent,
            accept_language,
            platform,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_user_agent_windows() {
        let ua = generate_user_agent("120.0.6099.129", "Win32");
        assert!(ua.contains("Windows NT 10.0"));
        assert!(ua.contains("Chrome/120.0.6099.129"));
        assert!(ua.contains("Safari/537.36"));
    }

    #[test]
    fn test_generate_user_agent_mac() {
        let ua = generate_user_agent("120.0.6099.129", "MacIntel");
        assert!(ua.contains("Macintosh"));
        assert!(ua.contains("Chrome/120.0.6099.129"));
    }

    #[test]
    fn test_generate_sec_ch_ua() {
        let header = generate_sec_ch_ua("120.0.6099.129");
        assert!(header.contains(r#""Chromium";v="120""#));
        assert!(header.contains(r#""Google Chrome";v="120""#));
    }

    #[test]
    fn test_generate_accept_language() {
        let lang = generate_accept_language("en-US");
        assert_eq!(lang, "en-US,en;q=0.9");

        let lang = generate_accept_language("fr-FR");
        assert_eq!(lang, "fr-FR,fr;q=0.9");
    }

    #[test]
    fn test_generate_sec_ch_ua_platform() {
        assert_eq!(generate_sec_ch_ua_platform("Win32"), r#""Windows""#);
        assert_eq!(generate_sec_ch_ua_platform("MacIntel"), r#""macOS""#);
        assert_eq!(generate_sec_ch_ua_platform("Linux x86_64"), r#""Linux""#);
    }
}
