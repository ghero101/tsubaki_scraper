// Application state for the Actix-web server

use reqwest::Client;
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Mutex;

#[derive(Debug, Default, Serialize, Clone)]
pub struct MetadataProgress {
    pub in_progress: bool,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub current_phase: Option<String>,
    pub total_pending: Option<i64>,
    pub processed_in_phase: usize,
    pub mangabaka_updated: usize,
    pub mal_updated: usize,
    pub anilist_updated: usize,
    pub merged_updated: usize,
    pub last_heartbeat: Option<i64>,
    pub error: Option<String>,
}

pub struct AppState {
    pub db: Mutex<Connection>,
    pub client: Client,
    pub _enhanced_client: crate::http_client::EnhancedHttpClient,
    pub metrics: crate::metrics::MetricsTracker,
    pub config: crate::config::Config,
    pub crawl_progress: Mutex<crate::crawler::CrawlProgress>,
    pub metadata_progress: Mutex<MetadataProgress>,
    pub metadata_cancel: Mutex<bool>,
}
