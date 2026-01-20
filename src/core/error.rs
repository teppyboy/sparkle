//! Error types for Sparkle (Playwright Rust)
//!
//! This module defines all error types that can occur during browser automation.
//! All errors implement the standard Error trait and can be converted to/from
//! other error types where appropriate.

use thiserror::Error;

/// The main error type for Sparkle operations
///
/// This enum represents all possible errors that can occur while using Sparkle.
/// It follows Playwright's error hierarchy while being idiomatic to Rust.
#[derive(Debug, Error)]
pub enum Error {
    /// Timeout occurred while waiting for an operation to complete
    #[error("Timeout: {message}")]
    Timeout {
        /// Description of what operation timed out
        message: String,
        /// Timeout duration in milliseconds
        timeout_ms: u64,
    },

    /// Element could not be found using the given selector
    #[error("Element not found: {selector}")]
    ElementNotFound {
        /// The selector that was used
        selector: String,
    },

    /// Multiple elements found when expecting a single element (strict mode)
    #[error("Multiple elements found matching selector: {selector} (found {count} elements)")]
    StrictModeViolation {
        /// The selector that matched multiple elements
        selector: String,
        /// Number of elements found
        count: usize,
    },

    /// Error from the underlying WebDriver implementation
    #[error("WebDriver error: {0}")]
    WebDriver(#[from] thirtyfour::error::WebDriverError),

    /// Network-related error
    #[error("Network error: {0}")]
    Network(String),

    /// Navigation failed
    #[error("Navigation failed: {0}")]
    Navigation(String),

    /// JSON serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid argument provided to a function
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Browser instance has been closed
    #[error("Browser has been closed")]
    BrowserClosed,

    /// Browser context has been closed
    #[error("Browser context has been closed")]
    ContextClosed,

    /// Page has been closed
    #[error("Page has been closed")]
    PageClosed,

    /// Frame has been detached from the page
    #[error("Frame has been detached")]
    FrameDetached,

    /// Element is not attached to the DOM
    #[error("Element is not attached to the DOM")]
    ElementNotAttached,

    /// Element is not visible
    #[error("Element is not visible: {selector}")]
    ElementNotVisible {
        /// The selector for the element
        selector: String,
    },

    /// Element is not enabled
    #[error("Element is not enabled: {selector}")]
    ElementNotEnabled {
        /// The selector for the element
        selector: String,
    },

    /// Element is not editable
    #[error("Element is not editable: {selector}")]
    ElementNotEditable {
        /// The selector for the element
        selector: String,
    },

    /// JavaScript evaluation failed
    #[error("JavaScript evaluation failed: {0}")]
    JsEvaluation(String),

    /// File operation error
    #[error("File error: {0}")]
    File(#[from] std::io::Error),

    /// URL parsing error
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    /// Resource download failed
    #[error("Download failed: {0}")]
    DownloadFailed(String),

    /// Screenshot operation failed
    #[error("Screenshot failed: {0}")]
    ScreenshotFailed(String),

    /// Video recording error
    #[error("Video recording error: {0}")]
    VideoRecording(String),

    /// Tracing error
    #[error("Tracing error: {0}")]
    Tracing(String),

    /// Browser type not supported
    #[error("Browser type not supported: {0}")]
    UnsupportedBrowser(String),

    /// Feature not implemented yet
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    /// Action on element failed
    #[error("Action failed: {0}")]
    ActionFailed(String),

    /// Internal error that shouldn't normally occur
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for Sparkle operations
///
/// This is a convenience alias for Result<T, Error> used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a timeout error
    pub fn timeout(message: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout {
            message: message.into(),
            timeout_ms,
        }
    }

    /// Create a timeout error from Duration
    pub fn timeout_duration(message: impl Into<String>, timeout: std::time::Duration) -> Self {
        Self::Timeout {
            message: message.into(),
            timeout_ms: timeout.as_millis() as u64,
        }
    }

    /// Create an element not found error
    pub fn element_not_found(selector: impl Into<String>) -> Self {
        Self::ElementNotFound {
            selector: selector.into(),
        }
    }

    /// Create a strict mode violation error
    pub fn strict_mode_violation(selector: impl Into<String>, count: usize) -> Self {
        Self::StrictModeViolation {
            selector: selector.into(),
            count,
        }
    }

    /// Create a network error
    pub fn network(message: impl Into<String>) -> Self {
        Self::Network(message.into())
    }

    /// Create a navigation error
    pub fn navigation(message: impl Into<String>) -> Self {
        Self::Navigation(message.into())
    }

    /// Create an invalid argument error
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::InvalidArgument(message.into())
    }

    /// Create a not implemented error
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::NotImplemented(feature.into())
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeout_error() {
        let err = Error::timeout("Element did not appear", 5000);
        assert!(matches!(err, Error::Timeout { .. }));
        assert!(err.to_string().contains("Timeout"));
        assert!(err.to_string().contains("Element did not appear"));
    }

    #[test]
    fn test_element_not_found_error() {
        let err = Error::element_not_found("button.submit");
        assert!(matches!(err, Error::ElementNotFound { .. }));
        assert!(err.to_string().contains("button.submit"));
    }

    #[test]
    fn test_strict_mode_violation() {
        let err = Error::strict_mode_violation("div", 5);
        assert!(matches!(err, Error::StrictModeViolation { .. }));
        assert!(err.to_string().contains("5 elements"));
    }
}
