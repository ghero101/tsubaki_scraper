#![allow(dead_code)]
use crate::models::{Chapter, Manga};
/// VIZ Media - Shonen Jump & More
/// Scrapes free chapters only (respects time-gated paywall)
use reqwest::Client;
use scraper::{Html, Selector};

// Re-use the comprehensive cleaning function from wp_manga
use crate::sources::wp_manga::clean_manga_title_public as clean_title;

const BASE_URL: &str = "https://www.viz.com";

pub async fn search_manga_with_urls(
    client: &Client,
    title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // Get the Shonen Jump chapters page which lists all series
    let search_url = if title.is_empty() {
        format!("{}/shonenjump/chapters", BASE_URL)
    } else {
        format!("{}/search?search={}", BASE_URL, urlencoding::encode(title))
    };

    let response = client.get(&search_url).send().await?.text().await?;
    let document = Html::parse_document(&response);

    let mut results = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();

    // Parse series links from the chapters page
    if let Ok(selector) = Selector::parse("a[href*='/shonenjump/chapters/']") {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                let url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("{}{}", BASE_URL, href)
                };

                // Avoid duplicates
                if seen_urls.contains(&url) {
                    continue;
                }
                seen_urls.insert(url.clone());

                let title_text_raw = element.text().collect::<String>().trim().to_string();

                // Apply comprehensive title cleaning
                let title_text = match clean_title(&title_text_raw) {
                    Some(cleaned) => cleaned,
                    None => continue, // Skip if filtering removes it
                };

                if url.contains("/chapters/") {
                    results.push((
                        Manga {
                            id: String::new(),
                            title: title_text,
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
                        },
                        url,
                    ));
                }

                if results.len() >= 10 {
                    break;
                }
            }
        }
    }

    Ok(results)
}

pub async fn get_chapters(
    client: &Client,
    series_url: &str,
) -> Result<Vec<Chapter>, reqwest::Error> {
    let response = client.get(series_url).send().await?.text().await?;
    let document = Html::parse_document(&response);

    let mut chapters = Vec::new();

    // Look for chapter links (only free chapters will be accessible)
    if let Ok(selector) = Selector::parse("a[href*='/chapter/'][href*='?action=read']") {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                let url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("{}{}", BASE_URL, href)
                };

                // Extract chapter number from URL or text
                let chapter_text = element.text().collect::<String>().trim().to_string();
                let chapter_num = if !chapter_text.is_empty() {
                    chapter_text
                } else if let Some(ch_num) = href.split("/chapter-").nth(1) {
                    format!("Chapter {}", ch_num.split('/').next().unwrap_or(""))
                } else {
                    format!("Chapter {}", chapters.len() + 1)
                };

                chapters.push(Chapter {
                    id: 0,
                    manga_source_data_id: 0,
                    chapter_number: chapter_num,
                    url,
                    scraped: false,
                });
            }
        }
    }

    Ok(chapters)
}
