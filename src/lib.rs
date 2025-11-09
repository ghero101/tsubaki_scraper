// Library interface for rust_manga_scraper
// This allows tests and external crates to use the scraper components

pub mod browser_client;
pub mod config;
pub mod http_client;
pub mod metrics;
pub mod models;
pub mod scraper;
pub mod source_utils;
pub mod sources;
pub mod sources_browser;

// Browser automation module (headless Chrome)
pub mod browser;

// Cloudflare bypass and anti-bot evasion
pub mod cloudflare_bypass;

// Database layer
pub mod db;

// Crawler for discovering manga
pub mod crawler;

// Task scheduler
pub mod scheduler;

// Metadata aggregation
pub mod metadata;

// Helper functions
pub mod helpers;

// Application state
pub mod app_state;
