use crate::models::{Chapter, Manga};
use crate::sources_browser::grimscans_browser;
use reqwest::Client;

const BASE_URL: &str = "https://grimscans.com";

/// GrimScans requires browser with Cloudflare bypass
/// Falls back to standard HTTP if browser fails
pub async fn search_manga_with_urls(
    client: &Client,
    _title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // Try browser first for Cloudflare bypass
    match grimscans_browser::search_manga_with_urls().await {
        Ok(results) if !results.is_empty() => {
            log::info!(
                "GrimScans: Successfully fetched {} manga using browser",
                results.len()
            );
            return Ok(results);
        }
        Ok(_) => log::warn!("GrimScans: Browser returned no results, trying fallback"),
        Err(e) => log::warn!("GrimScans: Browser method failed ({}), trying fallback", e),
    }

    // Fallback to standard HTTP
    crate::sources::wp_manga::search_manga_with_urls_base(client, BASE_URL).await
}

pub async fn get_chapters(
    client: &Client,
    series_url: &str,
) -> Result<Vec<Chapter>, reqwest::Error> {
    // Try browser first for Cloudflare bypass
    match grimscans_browser::get_chapters(series_url).await {
        Ok(chapters) if !chapters.is_empty() => {
            log::info!(
                "GrimScans: Successfully fetched {} chapters using browser",
                chapters.len()
            );
            return Ok(chapters);
        }
        Ok(_) => log::warn!("GrimScans: Browser returned no chapters, trying fallback"),
        Err(e) => log::warn!("GrimScans: Browser method failed ({}), trying fallback", e),
    }

    // Fallback to standard HTTP
    crate::sources::wp_manga::get_chapters_base(client, BASE_URL, series_url).await
}
