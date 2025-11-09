use crate::models::{Chapter, Manga};
use reqwest::Client;

#[allow(dead_code)]
const BASE_URL: &str = "https://mavintranslations.com";

#[allow(dead_code)]
pub async fn search_manga_with_urls(
    client: &Client,
    _title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    crate::sources::wp_manga::search_manga_first_page(client, BASE_URL).await
}

#[allow(dead_code)]
pub async fn get_chapters(
    client: &Client,
    series_url: &str,
) -> Result<Vec<Chapter>, reqwest::Error> {
    crate::sources::wp_manga::get_chapters_base(client, BASE_URL, series_url).await
}
