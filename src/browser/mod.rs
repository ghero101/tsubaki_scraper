//! Browser automation module for scraping JavaScript-heavy websites
//!
//! This module provides browser automation capabilities using headless Chrome
//! to scrape websites that require JavaScript execution or bypass anti-bot protection.
//!
//! # Example
//!
//! ```no_run
//! use rust_manga_scraper::browser::{BrowserConfig, BrowserManager, BrowserScraper};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a browser manager with default configuration
//! let manager = BrowserManager::new(BrowserConfig::default())?;
//!
//! // Create a new tab
//! let tab = manager.new_tab()?;
//!
//! // Create a scraper
//! let scraper = BrowserScraper::new(tab);
//!
//! // Navigate and extract HTML
//! scraper.navigate("https://example.com")?;
//! scraper.wait_for_selector("h1")?;
//! let html = scraper.get_html()?;
//!
//! println!("Extracted {} bytes of HTML", html.len());
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod manager;
pub mod scraper;

// Re-export main types for convenience
pub use config::BrowserConfig;
pub use manager::{BrowserError, BrowserManager};
pub use scraper::BrowserScraper;
