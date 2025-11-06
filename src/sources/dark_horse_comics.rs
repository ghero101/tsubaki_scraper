/// Dark Horse Comics - Free digital previews
use reqwest::Client;
use crate::models::{Manga, Chapter};
use scraper::{Html, Selector};

const BASE_URL: &str = "https://www.darkhorse.com";

pub async fn search_manga_with_urls(
    client: &Client,
    title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let search_url = if title.is_empty() {
        format!("{}/Comics", BASE_URL)
    } else {
        format!("{}/Search?q={}", BASE_URL, urlencoding::encode(title))
    };

    let response = client.get(&search_url).send().await?.text().await?;
    let document = Html::parse_document(&response);

    let mut results = Vec::new();

    // Parse from the carousel/list structure
    if let Ok(selector) = Selector::parse("#iscroll_n01_iscroll_list a, div.comic-item a, div.series-item a") {
        for element in document.select(&selector).take(10) {
            let url = element.value().attr("href")
                .map(|h| if h.starts_with("http") { h.to_string() } else { format!("{}{}", BASE_URL, h) })
                .unwrap_or_default();

            // Get title from h2 within the link
            let title_sel = Selector::parse("h2").ok();
            let title_text = if let Some(sel) = title_sel {
                element.select(&sel).next()
                    .map(|e| e.text().collect::<String>().trim().to_string())
                    .unwrap_or_else(|| element.text().collect::<String>().trim().to_string())
            } else {
                element.text().collect::<String>().trim().to_string()
            };

            if !url.is_empty() && !title_text.is_empty() {
                results.push((Manga {
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
                }, url));
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

    // Look for issue links or preview chapters
    if let Ok(selector) = Selector::parse("a[href*='/issue/'], a[href*='/Issues/'], div.issue-item a") {
        for element in document.select(&selector) {
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
                    format!("Issue {}", chapters.len() + 1)
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
