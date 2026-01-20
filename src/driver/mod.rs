//! WebDriver integration layer
//!
//! This module provides the bridge between Sparkle and thirtyfour (WebDriver).

pub mod capabilities;
pub mod chromedriver_process;
pub mod webdriver_adapter;

pub use capabilities::*;
pub use chromedriver_process::*;
pub use webdriver_adapter::*;
