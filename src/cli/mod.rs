//! CLI module for Sparkle
//!
//! Provides command-line tools for managing browsers and drivers.

pub mod download;
pub mod install;
pub mod list;
pub mod platform;
pub mod uninstall;

pub use download::Downloader;
pub use platform::Platform;
