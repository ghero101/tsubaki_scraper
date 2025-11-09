#![allow(dead_code)]
use crate::models::{Chapter, Manga};
use reqwest::Client;

const BASE_URL: &str = "https://www.webcomicsapp.com";

/// Webcomics - App-based platform
/// Note: May require app/API access for full functionality
pub async fn search_manga_with_urls(
    client: &Client,
    _title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // Webcomics is primarily app-based, web scraping may not work
    log::warn!("Webcomics is primarily an app-based platform");
    crate::sources::wp_manga::search_manga_first_page(client, BASE_URL).await
}

pub async fn get_chapters(
    client: &Client,
    series_url: &str,
) -> Result<Vec<Chapter>, reqwest::Error> {
    crate::sources::wp_manga::get_chapters_base(client, BASE_URL, series_url).await
}
