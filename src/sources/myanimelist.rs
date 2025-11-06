use reqwest::Client;
use crate::models::{Manga, Chapter};
use serde_json::Value;

const BASE_URL: &str = "https://myanimelist.net";
const API_URL: &str = "https://api.myanimelist.net/v2";

/// MyAnimeList - Metadata source (does not host chapters)
/// Note: This is a metadata/tracking source, not a reading source
pub async fn search_manga_with_urls(client: &Client, title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    if title.is_empty() {
        // Can't browse without search term on MAL
        log::info!("MyAnimeList requires a search term");
        return Ok(Vec::new());
    }

    // MAL API requires authentication, so we'll use the web search
    let search_url = format!("{}/manga.php?q={}&cat=manga", BASE_URL, urlencoding::encode(title));

    let response = match client.get(&search_url).send().await {
        Ok(r) => r.text().await?,
        Err(e) => {
            log::warn!("MyAnimeList search failed: {}", e);
            return Ok(Vec::new());
        }
    };

    // Parse search results (simplified - MAL has complex HTML)
    let document = scraper::Html::parse_document(&response);
    let mut results = Vec::new();

    if let Ok(selector) = scraper::Selector::parse("tr.ranking-list, div.js-categories-seasonal") {
        for element in document.select(&selector).take(10) {
            if let Ok(link_sel) = scraper::Selector::parse("a.hoverinfo_trigger") {
                if let Some(link) = element.select(&link_sel).next() {
                    if let Some(href) = link.value().attr("href") {
                        let url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            format!("{}{}", BASE_URL, href)
                        };

                        let title = link.text().collect::<String>().trim().to_string();

                        if !title.is_empty() {
                            results.push((Manga {
                                id: String::new(),
                                title,
                                alt_titles: None,
                                cover_url: None,
                                description: Some("Metadata source - no chapters available".to_string()),
                                tags: Some("MyAnimeList".to_string()),
                                rating: None,
                                monitored: None,
                                check_interval_secs: None,
                                discover_interval_secs: None,
                                last_chapter_check: None,
                                last_discover_check: None,
                            }, url));
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}

pub async fn get_chapters(_client: &Client, _series_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    // MyAnimeList is a metadata source, it doesn't host chapters
    Ok(Vec::new())
}
