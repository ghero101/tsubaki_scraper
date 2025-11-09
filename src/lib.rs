//! Tsubaki Scraper - A comprehensive manga aggregation and scraping library
//!
//! This library provides a complete manga scraping solution with support for 90+ sources,
//! sophisticated bot detection bypass, browser automation, and metadata aggregation.
//!
//! # Features
//!
//! - **Multi-source support**: 90+ manga sources including MangaDex, scanlation groups, and aggregators
//! - **Bot detection bypass**: Enhanced HTTP client with retry logic and browser automation fallback
//! - **Browser automation**: Headless Chrome for JavaScript-heavy sites
//! - **Cloudflare bypass**: TLS fingerprinting, JA3 randomization, CAPTCHA solver integration
//! - **Metadata aggregation**: Fetch and merge metadata from AniList, MyAnimeList, and Mangabaka
//! - **Chapter downloads**: ZIP creation with ComicInfo.xml metadata
//! - **Background monitoring**: Automatic chapter checking and discovery
//! - **Performance metrics**: Track success rates, latency, and errors per source
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use rust_manga_scraper::sources::mangadex;
//! use reqwest::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new();
//!
//!     // Search for manga
//!     let manga = mangadex::search_manga(
//!         &client,
//!         "one piece",
//!         mangadex::BASE_URL
//!     ).await?;
//!
//!     println!("Found {} manga", manga.len());
//!     Ok(())
//! }
//! ```
//!
//! # Module Organization
//!
//! - [`http_client`] - Enhanced HTTP client with bot detection bypass
//! - [`browser_client`] - Headless Chrome wrapper for browser automation
//! - [`sources`] - All manga source implementations (90+)
//! - [`models`] - Data structures (Manga, Chapter, Source enums)
//! - [`db`] - SQLite database operations
//! - [`scraper`] - Chapter download and ZIP creation
//! - [`crawler`] - Manga discovery and monitoring
//! - [`metadata`] - Metadata aggregation from multiple APIs
//! - [`helpers`] - Utility functions
//! - [`app_state`] - Application state for HTTP server
//!
//! # Architecture
//!
//! See `ARCHITECTURE.md` for detailed documentation on the codebase structure,
//! design decisions, and development guidelines.

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
