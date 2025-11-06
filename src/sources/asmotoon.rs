use reqwest::Client;
use crate::models::{Manga, Chapter};

const BASE_URL: &str = "https://asmotoon.com";

// NOTE: This site may require JavaScript rendering to load content properly.
// If the standard HTTP approach fails, use the browser client:
//
// use crate::browser_client::BrowserClient;
//
// pub async fn search_manga_with_urls_browser(title: &str) -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
//     let browser = BrowserClient::new().await?;
//     let url = format!("{}/?s={}&post_type=wp-manga", BASE_URL, title);
//     let html = browser.get_html(&url)?;
//     Ok(parse_search_page(&html))
// }

pub async fn search_manga_with_urls(client: &Client, _title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    crate::sources::wp_manga::search_manga_first_page(client, BASE_URL).await
}


pub async fn get_chapters(client: &Client, series_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    crate::sources::wp_manga::get_chapters_base(client, BASE_URL, series_url).await
}
