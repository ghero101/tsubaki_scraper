use reqwest::Client;
use scraper::{Html, Selector};
use crate::models::{Manga, Chapter};

const BASE_URL: &str = "https://asmotoon.com";

// NOTE: This site may require JavaScript rendering to load content properly.
// If the standard HTTP approach fails, use the browser client:
//
// use crate::browser_client::BrowserClient;
//
// pub async fn search_manga_with_urls_browser(title: &str) -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
//     let browser = BrowserClient::new()?;
//     let url = format!("{}/?s={}&post_type=wp-manga", BASE_URL, title);
//     let html = browser.get_html(&url)?;
//     Ok(parse_search_page(&html))
// }

pub async fn search_manga_with_urls(client: &Client, title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let url = format!("{}/?s={}&post_type=wp-manga", BASE_URL, title);

    // Enhanced headers to bypass bot detection
    let response = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Referer", BASE_URL)
        .send().await?.text().await?;

    Ok(parse_search_page(&response))
}

#[allow(dead_code)]
pub async fn search_all_manga_with_urls(client: &Client) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let mut page = 1u32;
    let mut out = Vec::new();
    loop {
        let url = format!("{}/?s=&post_type=wp-manga&paged={}", BASE_URL, page);
        let response = client.get(&url).send().await?.text().await?;
        let items = parse_search_page(&response);
        if items.is_empty() { break; }
        out.extend(items);
        page += 1;
        if page > 200 { break; }
    }
    Ok(out)
}

fn parse_search_page(html: &str) -> Vec<(Manga, String)> {
    let document = Html::parse_document(html);
    let selector = Selector::parse("div.page-item-detail").unwrap();
    let mut out = Vec::new();
    for element in document.select(&selector) {
        let title_selector = Selector::parse("h3 > a").unwrap();
        if let Some(title_element) = element.select(&title_selector).next() {
            let title = title_element.text().collect::<String>().trim().to_string();
            let series_url = title_element.value().attr("href").unwrap_or("").to_string();

            let cover_selector = Selector::parse("img").unwrap();
            let cover_url = element
                .select(&cover_selector)
                .next()
                .and_then(|e| e.value().attr("src").or_else(|| e.value().attr("data-src")))
                .map(|s| s.to_string());

            let manga = Manga {
                id: String::new(),
                title,
                alt_titles: None,
                cover_url,
                description: None,
                tags: None,
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
    out
}

pub async fn get_chapters(client: &Client, manga_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    // Enhanced headers to bypass bot detection
    let response = client.get(manga_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Referer", BASE_URL)
        .send().await?.text().await?;

    let document = Html::parse_document(&response);
    let selector = Selector::parse("li.wp-manga-chapter").unwrap();
    let mut chapters = Vec::new();

    for element in document.select(&selector) {
        let a_sel = Selector::parse("a").unwrap();
        if let Some(a) = element.select(&a_sel).next() {
            let chapter_title = a.text().collect::<String>().trim().to_string();
            if let Some(href) = a.value().attr("href") {
                chapters.push(Chapter {
                    id: 0,
                    manga_source_data_id: 0,
                    chapter_number: chapter_title,
                    url: href.to_string(),
                    scraped: false,
                });
            }
        }
    }

    Ok(chapters)
}
