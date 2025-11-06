#![allow(dead_code)]
use reqwest::Client;
use crate::models::{Manga, Chapter};
use scraper::{Html, Selector};

const BASE_URL: &str = "https://www.webtoons.com";

/// Webtoon - Free web platform with original content
/// Note: Only scrapes free/canvas content
pub async fn search_manga_with_urls(client: &Client, title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let search_url = if title.is_empty() {
        format!("{}/en/canvas/list?sortOrder=READ_COUNT", BASE_URL)
    } else {
        format!("{}/en/search?keyword={}", BASE_URL, urlencoding::encode(title))
    };

    let response = client.get(&search_url).send().await?.text().await?;
    let document = Html::parse_document(&response);

    let mut results = Vec::new();

    // Try multiple selectors for different page types
    let selectors = vec![
        "ul.card_lst li",
        "div.card_item",
        "li.challenge_item",
    ];

    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector).take(10) {
                if let Ok(link_sel) = Selector::parse("a") {
                    if let Some(link) = element.select(&link_sel).next() {
                        let url = link.value().attr("href")
                            .map(|h| if h.starts_with("http") { h.to_string() } else { format!("{}{}", BASE_URL, h) })
                            .unwrap_or_default();

                        if url.is_empty() { continue; }

                        let title = if let Some(title_attr) = link.value().attr("title") {
                            title_attr.to_string()
                        } else {
                            link.text().collect::<String>().trim().to_string()
                        };

                        if !title.is_empty() {
                            results.push((Manga {
                                id: String::new(),
                                title,
                                alt_titles: None,
                                cover_url: None,
                                description: None,
                                tags: None,
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

    Ok(results)
}

pub async fn get_chapters(client: &Client, series_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    let response = client.get(series_url).send().await?.text().await?;
    let document = Html::parse_document(&response);

    let mut chapters = Vec::new();

    if let Ok(selector) = Selector::parse("li._episodeItem, ul#_episodeList li") {
        for (idx, element) in document.select(&selector).enumerate() {
            if let Ok(link_sel) = Selector::parse("a") {
                if let Some(link) = element.select(&link_sel).next() {
                    let url = link.value().attr("href")
                        .map(|h| if h.starts_with("http") { h.to_string() } else { format!("{}{}", BASE_URL, h) })
                        .unwrap_or_default();

                    let title = if let Some(title_attr) = link.value().attr("title") {
                        title_attr.to_string()
                    } else {
                        let text = link.text().collect::<String>();
                        let trimmed = text.trim();
                        if trimmed.is_empty() {
                            format!("Episode {}", idx + 1)
                        } else {
                            trimmed.to_string()
                        }
                    };

                    if !url.is_empty() {
                        chapters.push(Chapter {
                            id: 0,
                            manga_source_data_id: 0,
                            chapter_number: title,
                            url,
                            scraped: false,
                        });
                    }
                }
            }
        }
    }

    Ok(chapters)
}
