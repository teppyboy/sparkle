//! WebDriver integration layer
//!
//! This module provides the bridge between Sparkle and thirtyfour (WebDriver).

pub mod capabilities;
pub mod webdriver_adapter;

pub use capabilities::*;
pub use webdriver_adapter::*;
