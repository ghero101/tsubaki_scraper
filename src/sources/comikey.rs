/// Comikey - Commercial/Paid Source
/// This source requires authentication/payment and is not available for free scraping
use reqwest::Client;
use crate::models::{Manga, Chapter};

const BASE_URL: &str = "https://comikey.com";

pub async fn search_manga_with_urls(
    _client: &Client,
    _title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // Commercial source - returns empty to prevent errors
    log::warn!("{} is a commercial/paid source and requires authentication", "Comikey");
    Ok(Vec::new())
}

pub async fn get_chapters(
    _client: &Client,
    _series_url: &str,
) -> Result<Vec<Chapter>, reqwest::Error> {
    Ok(Vec::new())
}
