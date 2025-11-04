use crate::models::Manga;
use reqwest::Client;
use scraper::{Html, Selector};

const BASE_URL: &str = "https://kdtnovels.com";

pub async fn search_manga_with_urls(
    client: &Client,
    title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // KDT Novels: treat entries as series; no chapter scraping for now
    let url = format!("{}/?s={}", BASE_URL, title);
    let response = client.get(&url).send().await?.text().await?;
    let document = Html::parse_document(&response);

    let mut out = Vec::new();
    let item_sel = Selector::parse("article").unwrap();
    let a_sel = Selector::parse("h2 a, .entry-title a").unwrap();
    let img_sel = Selector::parse("img").unwrap();

    for item in document.select(&item_sel) {
        if let Some(a) = item.select(&a_sel).next() {
            let title = a.text().collect::<String>().trim().to_string();
            let series_url = a.value().attr("href").unwrap_or("").to_string();
            let cover_url = item
                .select(&img_sel)
                .next()
                .and_then(|e| e.value().attr("src").or_else(|| e.value().attr("data-src")))
                .map(|s| s.to_string());

            let manga = Manga {
                id: String::new(),
                title,
                alt_titles: None,
                cover_url,
                description: None,
                tags: Some("Novel".to_string()),
                rating: None,
                monitored: None,
                check_interval_secs: None,
                discover_interval_secs: None,
                last_chapter_check: None,
                last_discover_check: None,
            };
            if !series_url.is_empty() {
                out.push((manga, series_url));
            }
        }
    }

    Ok(out)
}
