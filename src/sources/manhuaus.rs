use crate::models::{Chapter, Manga};
use reqwest::Client;

const BASE_URL: &str = "https://manhuaus.com";

pub async fn search_manga_with_urls(
    client: &Client,
    _title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    crate::sources::wp_manga::search_manga_with_urls_base(client, BASE_URL).await
}

pub async fn get_chapters(
    client: &Client,
    series_url: &str,
) -> Result<Vec<Chapter>, reqwest::Error> {
    crate::sources::wp_manga::get_chapters_base(client, BASE_URL, series_url).await
}
