//! Storage state types for saving and loading browser context state
//!
//! This module provides types compatible with Playwright's storage state format,
//! allowing you to save and restore cookies, localStorage, and sessionStorage.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents the complete storage state of a browser context
///
/// This matches Playwright's storage state format and can be serialized to/from JSON.
///
/// # Example
/// ```no_run
/// # use sparkle::core::StorageState;
/// # async fn example() -> sparkle::core::Result<()> {
/// # let context: sparkle::async_api::BrowserContext = todo!();
/// // Save storage state
/// let state = context.storage_state(Some("auth.json")).await?;
///
/// // Or get storage state without saving to file
/// let state = context.storage_state(None).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StorageState {
    /// List of cookies
    #[serde(default)]
    pub cookies: Vec<CookieState>,

    /// List of origins with their localStorage and sessionStorage
    #[serde(default)]
    pub origins: Vec<OriginState>,
}

impl StorageState {
    /// Create a new empty storage state
    pub fn new() -> Self {
        Self::default()
    }

    /// Load storage state from a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to the JSON file
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed
    pub fn from_file(path: impl Into<PathBuf>) -> crate::core::Result<Self> {
        let path = path.into();
        let content = std::fs::read_to_string(&path).map_err(|e| {
            crate::core::Error::ActionFailed(format!(
                "Failed to read storage state from {}: {}",
                path.display(),
                e
            ))
        })?;

        Self::from_json(&content)
    }

    /// Parse storage state from a JSON string
    ///
    /// # Arguments
    /// * `json` - JSON string containing storage state
    ///
    /// # Errors
    /// Returns an error if the JSON cannot be parsed
    pub fn from_json(json: &str) -> crate::core::Result<Self> {
        serde_json::from_str(json).map_err(|e| {
            crate::core::Error::ActionFailed(format!("Failed to parse storage state JSON: {}", e))
        })
    }

    /// Save storage state to a JSON file
    ///
    /// # Arguments
    /// * `path` - Path to save the JSON file
    ///
    /// # Errors
    /// Returns an error if the file cannot be written
    pub fn to_file(&self, path: impl Into<PathBuf>) -> crate::core::Result<()> {
        let path = path.into();
        let json = self.to_json()?;
        std::fs::write(&path, json).map_err(|e| {
            crate::core::Error::ActionFailed(format!(
                "Failed to write storage state to {}: {}",
                path.display(),
                e
            ))
        })?;
        Ok(())
    }

    /// Convert storage state to a JSON string
    ///
    /// # Errors
    /// Returns an error if serialization fails
    pub fn to_json(&self) -> crate::core::Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| {
            crate::core::Error::ActionFailed(format!("Failed to serialize storage state: {}", e))
        })
    }
}

/// Represents a single cookie in the storage state
///
/// This matches Playwright's cookie format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CookieState {
    /// Cookie name
    pub name: String,

    /// Cookie value
    pub value: String,

    /// Cookie domain (e.g., ".example.com")
    pub domain: String,

    /// Cookie path (e.g., "/")
    pub path: String,

    /// Expiration time in Unix seconds. Use -1 for session cookies.
    pub expires: f64,

    /// Whether the cookie is HTTP-only
    pub http_only: bool,

    /// Whether the cookie is secure (HTTPS only)
    pub secure: bool,

    /// SameSite attribute
    pub same_site: SameSite,
}

/// SameSite cookie attribute
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SameSite {
    /// Strict mode - cookie only sent in first-party context
    Strict,
    /// Lax mode - cookie sent with top-level navigation
    Lax,
    /// None mode - cookie sent in all contexts (requires Secure)
    None,
}

impl Default for SameSite {
    fn default() -> Self {
        Self::Lax
    }
}

/// Represents storage (localStorage and sessionStorage) for a single origin
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginState {
    /// Origin URL (e.g., "https://example.com")
    pub origin: String,

    /// localStorage entries
    #[serde(default)]
    pub local_storage: Vec<NameValue>,

    /// sessionStorage entries (note: sessionStorage is typically ephemeral)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub session_storage: Vec<NameValue>,
}

/// A name-value pair for storage entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameValue {
    /// Entry name/key
    pub name: String,

    /// Entry value
    pub value: String,
}

/// Source for loading storage state into a browser context
///
/// This can be either a file path or an inline StorageState object.
#[derive(Debug, Clone)]
pub enum StorageStateSource {
    /// Load from a file path
    Path(String),

    /// Use an inline StorageState object
    State(StorageState),
}

impl From<String> for StorageStateSource {
    fn from(path: String) -> Self {
        Self::Path(path)
    }
}

impl From<&str> for StorageStateSource {
    fn from(path: &str) -> Self {
        Self::Path(path.to_string())
    }
}

impl From<StorageState> for StorageStateSource {
    fn from(state: StorageState) -> Self {
        Self::State(state)
    }
}

impl StorageStateSource {
    /// Load the storage state, reading from file if necessary
    pub fn load(self) -> crate::core::Result<StorageState> {
        match self {
            Self::Path(path) => StorageState::from_file(path),
            Self::State(state) => Ok(state),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_state_serialization() {
        let state = StorageState {
            cookies: vec![CookieState {
                name: "session".to_string(),
                value: "abc123".to_string(),
                domain: ".example.com".to_string(),
                path: "/".to_string(),
                expires: -1.0,
                http_only: true,
                secure: true,
                same_site: SameSite::Lax,
            }],
            origins: vec![OriginState {
                origin: "https://example.com".to_string(),
                local_storage: vec![NameValue {
                    name: "user".to_string(),
                    value: "john".to_string(),
                }],
                session_storage: vec![],
            }],
        };

        let json = state.to_json().unwrap();
        assert!(json.contains("\"cookies\""));
        assert!(json.contains("\"origins\""));
        assert!(json.contains("\"session\""));
        assert!(json.contains("\"abc123\""));

        let parsed = StorageState::from_json(&json).unwrap();
        assert_eq!(parsed.cookies.len(), 1);
        assert_eq!(parsed.cookies[0].name, "session");
        assert_eq!(parsed.origins.len(), 1);
        assert_eq!(parsed.origins[0].origin, "https://example.com");
    }

    #[test]
    fn test_same_site_serialization() {
        let json = serde_json::to_string(&SameSite::Strict).unwrap();
        assert_eq!(json, "\"Strict\"");

        let json = serde_json::to_string(&SameSite::Lax).unwrap();
        assert_eq!(json, "\"Lax\"");

        let json = serde_json::to_string(&SameSite::None).unwrap();
        assert_eq!(json, "\"None\"");
    }

    #[test]
    fn test_empty_storage_state() {
        let state = StorageState::default();
        let json = state.to_json().unwrap();
        let parsed = StorageState::from_json(&json).unwrap();
        assert_eq!(parsed.cookies.len(), 0);
        assert_eq!(parsed.origins.len(), 0);
    }

    #[test]
    fn test_storage_state_source() {
        let source: StorageStateSource = "auth.json".into();
        assert!(matches!(source, StorageStateSource::Path(_)));

        let state = StorageState::default();
        let source: StorageStateSource = state.into();
        assert!(matches!(source, StorageStateSource::State(_)));
    }

    #[test]
    fn test_playwright_compatible_json_format() {
        // This test ensures our JSON format matches Playwright's exactly
        let state = StorageState {
            cookies: vec![
                CookieState {
                    name: "auth_token".to_string(),
                    value: "secret123".to_string(),
                    domain: ".github.com".to_string(),
                    path: "/".to_string(),
                    expires: 1735689600.0, // 2025-01-01
                    http_only: true,
                    secure: true,
                    same_site: SameSite::Lax,
                },
                CookieState {
                    name: "session".to_string(),
                    value: "xyz789".to_string(),
                    domain: "github.com".to_string(),
                    path: "/".to_string(),
                    expires: -1.0, // session cookie
                    http_only: false,
                    secure: true,
                    same_site: SameSite::Strict,
                },
            ],
            origins: vec![OriginState {
                origin: "https://github.com".to_string(),
                local_storage: vec![
                    NameValue {
                        name: "user_id".to_string(),
                        value: "12345".to_string(),
                    },
                    NameValue {
                        name: "preferences".to_string(),
                        value: r#"{"theme":"dark","lang":"en"}"#.to_string(),
                    },
                ],
                session_storage: vec![NameValue {
                    name: "temp_data".to_string(),
                    value: "temporary".to_string(),
                }],
            }],
        };

        let json = state.to_json().unwrap();

        // Verify camelCase field names (Playwright format)
        assert!(json.contains("\"cookies\""));
        assert!(json.contains("\"origins\""));
        assert!(json.contains("\"localStorage\""));
        assert!(json.contains("\"sessionStorage\""));
        assert!(json.contains("\"httpOnly\""));
        assert!(json.contains("\"sameSite\""));

        // Verify it can be parsed back
        let parsed = StorageState::from_json(&json).unwrap();
        assert_eq!(parsed.cookies.len(), 2);
        assert_eq!(parsed.cookies[0].name, "auth_token");
        assert_eq!(parsed.cookies[1].same_site, SameSite::Strict);
        assert_eq!(parsed.origins[0].local_storage.len(), 2);
        assert_eq!(parsed.origins[0].session_storage.len(), 1);
    }

    #[test]
    fn test_cookie_same_site_variants() {
        // Test all SameSite variants
        let state = StorageState {
            cookies: vec![
                CookieState {
                    name: "cookie1".to_string(),
                    value: "val1".to_string(),
                    domain: "example.com".to_string(),
                    path: "/".to_string(),
                    expires: -1.0,
                    http_only: false,
                    secure: false,
                    same_site: SameSite::Strict,
                },
                CookieState {
                    name: "cookie2".to_string(),
                    value: "val2".to_string(),
                    domain: "example.com".to_string(),
                    path: "/".to_string(),
                    expires: -1.0,
                    http_only: false,
                    secure: false,
                    same_site: SameSite::Lax,
                },
                CookieState {
                    name: "cookie3".to_string(),
                    value: "val3".to_string(),
                    domain: "example.com".to_string(),
                    path: "/".to_string(),
                    expires: -1.0,
                    http_only: false,
                    secure: true,
                    same_site: SameSite::None,
                },
            ],
            origins: vec![],
        };

        let json = state.to_json().unwrap();
        let parsed = StorageState::from_json(&json).unwrap();

        assert_eq!(parsed.cookies[0].same_site, SameSite::Strict);
        assert_eq!(parsed.cookies[1].same_site, SameSite::Lax);
        assert_eq!(parsed.cookies[2].same_site, SameSite::None);
    }

    #[test]
    fn test_session_storage_serialization() {
        let state = StorageState {
            cookies: vec![],
            origins: vec![OriginState {
                origin: "https://example.com".to_string(),
                local_storage: vec![NameValue {
                    name: "persistent".to_string(),
                    value: "data".to_string(),
                }],
                session_storage: vec![NameValue {
                    name: "temporary".to_string(),
                    value: "session_data".to_string(),
                }],
            }],
        };

        let json = state.to_json().unwrap();
        assert!(json.contains("\"sessionStorage\""));
        assert!(json.contains("\"temporary\""));

        let parsed = StorageState::from_json(&json).unwrap();
        assert_eq!(parsed.origins[0].session_storage.len(), 1);
        assert_eq!(parsed.origins[0].session_storage[0].name, "temporary");
    }

    #[test]
    fn test_empty_session_storage_omitted() {
        // When sessionStorage is empty, it should still be valid but can be omitted
        let state = StorageState {
            cookies: vec![],
            origins: vec![OriginState {
                origin: "https://example.com".to_string(),
                local_storage: vec![],
                session_storage: vec![],
            }],
        };

        let json = state.to_json().unwrap();
        let parsed = StorageState::from_json(&json).unwrap();
        assert_eq!(parsed.origins[0].session_storage.len(), 0);
    }
}
