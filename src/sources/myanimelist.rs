#![allow(dead_code)]
use reqwest::Client;
use crate::models::{Manga, Chapter};

const BASE_URL: &str = "https://myanimelist.net";
const API_URL: &str = "https://api.myanimelist.net/v2";

/// MyAnimeList - Metadata source (does not host chapters)
/// Note: This is a metadata/tracking source, not a reading source
pub async fn search_manga_with_urls(client: &Client, title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // If no search term, get top manga instead
    let search_url = if title.is_empty() {
        format!("{}/topmanga.php?limit=0", BASE_URL)
    } else {
        format!("{}/manga.php?q={}&cat=manga", BASE_URL, urlencoding::encode(title))
    };

    let response = match client.get(&search_url).send().await {
        Ok(r) => r.text().await?,
        Err(e) => {
            log::warn!("MyAnimeList search failed: {}", e);
            return Ok(Vec::new());
        }
    };

    // Parse search results
    let document = scraper::Html::parse_document(&response);
    let mut results = Vec::new();

    // Try multiple selectors for different page types
    let selectors = vec![
        ("tr.ranking-list", "a.hoverinfo_trigger"),
        ("div.js-categories-seasonal", "a.link-title"),
        ("div.list-table", "a.hoverinfo_trigger"),
    ];

    for (container_sel, link_sel) in selectors {
        if let Ok(container) = scraper::Selector::parse(container_sel) {
            if let Ok(link_selector) = scraper::Selector::parse(link_sel) {
                for element in document.select(&container).take(10) {
                    if let Some(link) = element.select(&link_selector).next() {
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
                if !results.is_empty() {
                    break;
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
