use reqwest::Client;
use crate::models::{Manga, Chapter};
use crate::sources_browser::rizzfables_browser;

const BASE_URL: &str = "https://rizzfables.com";

/// RizzFables requires browser with Cloudflare bypass
/// Falls back to standard HTTP if browser fails
pub async fn search_manga_with_urls(client: &Client, _title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // Try browser first for Cloudflare bypass
    match rizzfables_browser::search_manga_with_urls().await {
        Ok(results) if !results.is_empty() => {
            log::info!("RizzFables: Successfully fetched {} manga using browser", results.len());
            return Ok(results);
        }
        Ok(_) => log::warn!("RizzFables: Browser returned no results, trying fallback"),
        Err(e) => log::warn!("RizzFables: Browser method failed ({}), trying fallback", e),
    }

    // Fallback to standard HTTP
    crate::sources::wp_manga::search_manga_first_page(client, BASE_URL).await
}

pub async fn get_chapters(client: &Client, series_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    // Try browser first for Cloudflare bypass
    match rizzfables_browser::get_chapters(series_url).await {
        Ok(chapters) if !chapters.is_empty() => {
            log::info!("RizzFables: Successfully fetched {} chapters using browser", chapters.len());
            return Ok(chapters);
        }
        Ok(_) => log::warn!("RizzFables: Browser returned no chapters, trying fallback"),
        Err(e) => log::warn!("RizzFables: Browser method failed ({}), trying fallback", e),
    }

    // Fallback to standard HTTP
    crate::sources::wp_manga::get_chapters_base(client, BASE_URL, series_url).await
}
