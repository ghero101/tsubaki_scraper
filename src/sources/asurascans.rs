use reqwest::Client;
use scraper::{Html, Selector};
use crate::models::{Manga, Chapter};
use regex::Regex;

const BASE_URL: &str = "https://asuracomic.net";

/// Asura Scans - Next.js site with HTML-rendered data
pub async fn search_manga_with_urls(client: &Client, _title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let html = client.get(BASE_URL).send().await?.text().await?;
    let document = Html::parse_document(&html);

    // Select all series links: <a href="/series/slug">Title</a>
    let link_selector = Selector::parse("a[href*='/series/']").unwrap();

    let mut results = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();

    for element in document.select(&link_selector) {
        if let Some(href) = element.value().attr("href") {
            // Only process /series/ links that end with the slug (no chapter or additional paths)
            if href.contains("/series/") && !href.contains("/chapter/") {
                // Extract the slug part after /series/
                let slug_part = href.split("/series/").nth(1).unwrap_or("");

                // Skip if there are additional path segments after the slug
                if slug_part.contains('/') {
                    continue;
                }

                let series_url = if href.starts_with("http") {
                    href.to_string()
                } else if href.starts_with("/") {
                    format!("{}{}", BASE_URL, href)
                } else {
                    format!("{}/series/{}", BASE_URL, href)
                };

                // Skip duplicates
                if seen_urls.contains(&series_url) {
                    continue;
                }
                seen_urls.insert(series_url.clone());

                // Extract title from link text
                let raw_title = element.text().collect::<String>().trim().to_string();

                // Skip links with no text content at all
                if raw_title.is_empty() {
                    continue;
                }

                // Clean up title - remove common unwanted patterns
                // Remove "MANHWA", "MANGA", "Chapter X", ratings, etc.
                let mut title = raw_title.split("Chapter").next().unwrap_or(&raw_title).trim().to_string();
                title = title.replace("MANHWA", "").replace("MANGA", "").trim().to_string();

                // Remove trailing numbers that might be ratings (e.g., "9.5")
                let title_clean = Regex::new(r"\s*\d+\.?\d*\s*$").unwrap();
                title = title_clean.replace(&title, "").trim().to_string();

                // Skip if title became empty after cleaning, is too short, or too long
                if title.is_empty() || title.len() < 3 || title.len() > 100 {
                    continue;
                }

                let manga = Manga {
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
                };

                results.push((manga, series_url));

                if results.len() >= 10 {
                    break;
                }
            }
        }
    }

    log::debug!("AsuraScans: Found {} manga from homepage", results.len());
    Ok(results)
}

pub async fn get_chapters(client: &Client, series_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    let html = client.get(series_url).send().await?.text().await?;

    log::debug!("AsuraScans: Fetched {} bytes from {}", html.len(), series_url);

    // Extract chapter URLs using regex: href="series-slug/chapter/NUMBER"
    let chapter_regex = Regex::new(r#"href="([^"]*chapter/(\d+)[^"]*)""#).unwrap();

    let mut chapter_map = std::collections::HashMap::new();

    for cap in chapter_regex.captures_iter(&html) {
        let href = &cap[1];
        let chapter_num = &cap[2];

        // Build full URL
        let chapter_url = if href.starts_with("http") {
            href.to_string()
        } else if href.starts_with("/") {
            format!("{}{}", BASE_URL, href)
        } else {
            format!("{}/series/{}", BASE_URL, href)
        };

        // Use chapter number as key to deduplicate
        let num: u32 = chapter_num.parse().unwrap_or(0);
        chapter_map.insert(num, chapter_url);
    }

    // Convert to sorted vector
    let mut chapters: Vec<_> = chapter_map.into_iter().collect();
    chapters.sort_by_key(|&(num, _)| num);

    let result: Vec<Chapter> = chapters
        .into_iter()
        .map(|(num, url)| Chapter {
            id: 0,
            manga_source_data_id: 0,
            chapter_number: num.to_string(),
            url,
            scraped: false,
        })
        .collect();

    log::debug!("AsuraScans: Found {} chapters", result.len());
    Ok(result)
}
