/// Template for commercial/unavailable sources
/// These sources require authentication, payment, or are otherwise unavailable for scraping
use reqwest::Client;
use crate::models::{Manga, Chapter};

pub async fn search_manga_with_urls(
    _client: &Client,
    _title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // Return empty result for commercial/unavailable sources
    // This prevents errors while maintaining the interface
    Ok(Vec::new())
}

pub async fn get_chapters(
    _client: &Client,
    _series_url: &str,
) -> Result<Vec<Chapter>, reqwest::Error> {
    Ok(Vec::new())
}
