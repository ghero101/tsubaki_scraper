//! Application state for the Actix-web server
//!
//! This module defines the shared state used across all HTTP handlers.
//! The `AppState` struct is wrapped in `web::Data` and provides thread-safe
//! access to the database, HTTP client, and other shared resources.
//!
//! # Structure
//!
//! - `AppState`: Main application state with database connection, clients, and configuration
//! - `MetadataProgress`: Progress tracking for metadata synchronization operations

use deadpool_postgres::Pool;
use reqwest::Client;
use serde::Serialize;
use std::sync::{Arc, Mutex};

/// Progress tracking for metadata synchronization operations
#[derive(Debug, Default, Serialize, Clone)]
pub struct MetadataProgress {
    /// Whether metadata sync is currently running
    pub in_progress: bool,
    /// Unix timestamp when sync started
    pub started_at: Option<i64>,
    /// Unix timestamp when sync finished
    pub finished_at: Option<i64>,
    /// Current phase of the sync process
    pub current_phase: Option<String>,
    /// Total number of manga pending processing
    pub total_pending: Option<i64>,
    /// Number processed in current phase
    pub processed_in_phase: usize,
    /// Number of manga updated from Mangabaka
    pub mangabaka_updated: usize,
    /// Number of manga updated from MyAnimeList
    pub mal_updated: usize,
    /// Number of manga updated from AniList
    pub anilist_updated: usize,
    /// Number of manga with merged metadata
    pub merged_updated: usize,
    /// Last heartbeat timestamp for progress monitoring
    pub last_heartbeat: Option<i64>,
    /// Error message if sync failed
    pub error: Option<String>,
}

/// Shared application state for Actix-web handlers
///
/// This struct is wrapped in `web::Data` and shared across all HTTP request handlers.
/// All mutable state is protected by `Mutex` for thread-safety.
pub struct AppState {
    /// PostgreSQL database connection pool (inherently thread-safe)
    pub pool: Pool,
    /// Standard reqwest HTTP client
    pub client: Client,
    /// Enhanced HTTP client with bot detection bypass
    pub _enhanced_client: crate::http_client::EnhancedHttpClient,
    /// Metrics tracker for monitoring source performance
    pub metrics: crate::metrics::MetricsTracker,
    /// Application configuration
    pub config: crate::config::Config,
    /// Progress tracking for crawler operations
    pub crawl_progress: Mutex<crate::crawler::CrawlProgress>,
    /// Progress tracking for metadata sync operations
    pub metadata_progress: Mutex<MetadataProgress>,
    /// Flag to cancel ongoing metadata sync
    pub metadata_cancel: Mutex<bool>,
    /// Browser manager for Cloudflare-protected sources (optional)
    pub browser_manager: Option<Arc<crate::browser::BrowserManager>>,
}
