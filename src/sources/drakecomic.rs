use reqwest::Client;
use scraper::{Html, Selector};
use crate::models::{Manga, Chapter};

const BASE_URL: &str = "https://drakecomic.org";

pub async fn search_manga_with_urls(client: &Client, title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let url = format!("{}/?s={}&post_type=wp-manga", BASE_URL, title);
    let response = client.get(&url).send().await?.text().await?;
    let document = Html::parse_document(&response);

    let selectors = [
        "div.page-item-detail",
        "div.manga-item",
    ];

    let mut out = Vec::new();
    for sel in &selectors {
        if let Ok(selector) = Selector::parse(sel) {
            for element in document.select(&selector) {
                let title_selector = Selector::parse("h3 > a, a.item-title").unwrap();
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
            if !out.is_empty() { break; }
        }
    }

    Ok(out)
}

pub async fn get_chapters(client: &Client, manga_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    let response = client.get(manga_url).send().await?.text().await?;
    let document = Html::parse_document(&response);

    let selectors = [
        "li.wp-manga-chapter",
        "ul.main.version-chap li",
    ];

    let mut chapters = Vec::new();
    'outer: for sel in &selectors {
        if let Ok(selector) = Selector::parse(sel) {
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
            if !chapters.is_empty() { break 'outer; }
        }
    }

    Ok(chapters)
}
