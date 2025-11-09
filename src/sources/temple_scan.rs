use crate::models::{Chapter, Manga};
use crate::sources_browser::templescan_browser;
use reqwest::Client;

const BASE_URL: &str = "https://templetoons.com";

/// TempleScan may require browser for JS-rendered content
/// Falls back to standard HTTP if browser fails
pub async fn search_manga_with_urls(
    client: &Client,
    _title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // Try browser first for JS rendering
    match templescan_browser::search_manga_with_urls().await {
        Ok(results) if !results.is_empty() => {
            log::info!(
                "TempleScan: Successfully fetched {} manga using browser",
                results.len()
            );
            return Ok(results);
        }
        Ok(_) => log::warn!("TempleScan: Browser returned no results, trying fallback"),
        Err(e) => log::warn!("TempleScan: Browser method failed ({}), trying fallback", e),
    }

    // Fallback to standard HTTP
    crate::sources::wp_manga::search_manga_first_page(client, BASE_URL).await
}

pub async fn get_chapters(
    client: &Client,
    series_url: &str,
) -> Result<Vec<Chapter>, reqwest::Error> {
    // Try browser first for JS rendering
    match templescan_browser::get_chapters(series_url).await {
        Ok(chapters) if !chapters.is_empty() => {
            log::info!(
                "TempleScan: Successfully fetched {} chapters using browser",
                chapters.len()
            );
            return Ok(chapters);
        }
        Ok(_) => log::warn!("TempleScan: Browser returned no chapters, trying fallback"),
        Err(e) => log::warn!("TempleScan: Browser method failed ({}), trying fallback", e),
    }

    // Fallback to standard HTTP
    crate::sources::wp_manga::get_chapters_base(client, BASE_URL, series_url).await
}
