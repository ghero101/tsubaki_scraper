#![allow(dead_code)]
use reqwest::Client;
use crate::models::{Manga, Chapter};
use scraper::{Html, Selector};

const BASE_URL: &str = "https://tapas.io";

/// Tapas - Free web platform (only free content)
pub async fn search_manga_with_urls(client: &Client, title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let search_url = if title.is_empty() {
        format!("{}/comics", BASE_URL)
    } else {
        format!("{}/search?t=COMICS&q={}", BASE_URL, urlencoding::encode(title))
    };

    let response = client.get(&search_url).send().await?.text().await?;
    let document = Html::parse_document(&response);

    let mut results = Vec::new();

    if let Ok(selector) = Selector::parse("div.content__item, a.content-item") {
        for element in document.select(&selector).take(10) {
            let url = element.value().attr("href")
                .map(|h| if h.starts_with("http") { h.to_string() } else { format!("{}{}", BASE_URL, h) })
                .unwrap_or_default();

            let title_sel = Selector::parse("div.title, div.title__body, a.title").ok();
            let title = title_sel.and_then(|sel| element.select(&sel).next())
                .map(|e| e.text().collect::<String>().trim().to_string())
                .unwrap_or_default();

            if !url.is_empty() && !title.is_empty() {
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

    Ok(results)
}

pub async fn get_chapters(client: &Client, series_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    let response = client.get(series_url).send().await?.text().await?;
    let document = Html::parse_document(&response);

    let mut chapters = Vec::new();

    if let Ok(selector) = Selector::parse("div.episode-item, a.episode-link") {
        for element in document.select(&selector) {
            let url = element.value().attr("href")
                .map(|h| if h.starts_with("http") { h.to_string() } else { format!("{}{}", BASE_URL, h) })
                .unwrap_or_default();

            let title = element.text().collect::<String>().trim().to_string();

            if !url.is_empty() && !title.is_empty() {
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

    Ok(chapters)
}
