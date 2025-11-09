#![allow(dead_code)]
use crate::models::{Chapter, Manga};
/// Seven Seas Entertainment - Free preview chapters
use reqwest::Client;
use scraper::{Html, Selector};

const BASE_URL: &str = "https://sevenseasentertainment.com";

pub async fn search_manga_with_urls(
    client: &Client,
    title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let search_url = if title.is_empty() {
        format!("{}/series/", BASE_URL)
    } else {
        format!("{}/search/?q={}", BASE_URL, urlencoding::encode(title))
    };

    let response = client.get(&search_url).send().await?.text().await?;
    let document = Html::parse_document(&response);

    let mut results = Vec::new();

    // Parse series listings
    let selectors = vec![
        "div.series-card a",
        "article.book a[href*='/books/']",
        "div.book-item a",
        "a[href*='/books/']",
    ];

    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            for element in document.select(&selector).take(10) {
                let url = element
                    .value()
                    .attr("href")
                    .map(|h| {
                        if h.starts_with("http") {
                            h.to_string()
                        } else {
                            format!("{}{}", BASE_URL, h)
                        }
                    })
                    .unwrap_or_default();

                let title_text = element.text().collect::<String>().trim().to_string();

                if !url.is_empty() && !title_text.is_empty() && url.contains("/books/") {
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
            }

            if !results.is_empty() {
                break;
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

    // Look for volume/chapter links
    if let Ok(selector) =
        Selector::parse("a[href*='/volume/'], div.volume-item a, div.chapter-item a")
    {
        for (idx, element) in document.select(&selector).enumerate() {
            if let Some(href) = element.value().attr("href") {
                let url = if href.starts_with("http") {
                    href.to_string()
                } else {
                    format!("{}{}", BASE_URL, href)
                };

                let chapter_text = element.text().collect::<String>().trim().to_string();
                let chapter_num = if !chapter_text.is_empty() {
                    chapter_text
                } else {
                    format!("Volume {}", idx + 1)
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
