use crate::models::{Chapter, Manga};
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;

const BASE_URL: &str = "https://kagane.org";

/// Best-effort search over Kagane's public search page.
/// Note: Kagane is a Next.js app; much of the content is client-rendered.
/// This parser attempts to extract SSR-available links and metadata if present.
pub async fn search_manga_with_urls(
    client: &Client,
    title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let encoded = title.replace(' ', "%20");
    let url = format!("{}/search?query={}", BASE_URL, encoded);
    let response = client
        .get(&url)
        .header("User-Agent", "rust_manga_scraper/0.1.0")
        .header("Cookie", "nsfw=true; consent=true")
        .send()
        .await?
        .text()
        .await?;
    let document = Html::parse_document(&response);

    // Attempt to parse Next.js data if present
    let mut out = Vec::new();
    if let Ok(script_sel) = Selector::parse("script#__NEXT_DATA__") {
        if let Some(script) = document.select(&script_sel).next() {
            let txt = script.text().collect::<String>();
            if let Ok(json) = serde_json::from_str::<Value>(&txt) {
                // Try to find series list in props
                if let Some(arr) = json.pointer("/props/pageProps/series")
                    .and_then(|v| v.as_array()) {
                    for item in arr {
                        let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
                        if !slug.is_empty() {
                            let series_url = format!("{}/series/{}", BASE_URL, slug);
                            out.push((Manga { id: String::new(), title, alt_titles: None, cover_url: None, description: None, tags: None, rating: None, monitored: None, check_interval_secs: None, discover_interval_secs: None, last_chapter_check: None, last_discover_check: None }, series_url));
                        }
                    }
                }
            }
        }
    }

    // Fallback to anchors
    let a_sel = Selector::parse("a").unwrap();
    for a in document.select(&a_sel) {
        if let Some(href) = a.value().attr("href") {
            if href.starts_with("/series/") {
                let title_text = a.text().collect::<String>().trim().to_string();
                // Try to find a nearby image as cover
                let mut cover_url: Option<String> = None;
                if let Some(parent) = a.parent() {
                    for node in parent.descendants() {
                        if let Some(el) = scraper::ElementRef::wrap(node) {
                            if el.value().name() == "img" {
                                if let Some(src) = el
                                    .value()
                                    .attr("src")
                                    .or_else(|| el.value().attr("data-src"))
                                {
                                    if src.starts_with("http") {
                                        cover_url = Some(src.to_string());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                let m = Manga {
                    id: String::new(),
                    title: if title_text.is_empty() {
                        href.trim_start_matches("/series/").to_string()
                    } else {
                        title_text
                    },
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
                out.push((m, format!("{}{}", BASE_URL, href)));
            }
        }
    }

    Ok(out)
}

/// Fetch chapters for a Kagane series page by scanning for links to reader pages.
pub async fn get_chapters(
    client: &Client,
    series_url: &str,
) -> Result<Vec<Chapter>, reqwest::Error> {
    let response = client
        .get(series_url)
        .header("User-Agent", "rust_manga_scraper/0.1.0")
        .header("Cookie", "nsfw=true; consent=true")
        .send()
        .await?
        .text()
        .await?;
    let document = Html::parse_document(&response);

    let mut chapters = Vec::new();

    // Try Next.js data first
    if let Ok(script_sel) = Selector::parse("script#__NEXT_DATA__") {
        if let Some(script) = document.select(&script_sel).next() {
            let txt = script.text().collect::<String>();
            if let Ok(json) = serde_json::from_str::<Value>(&txt) {
                if let Some(arr) = json.pointer("/props/pageProps/chapters").and_then(|v| v.as_array()) {
                    for c in arr {
                        let title = c.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        // Prefer absolute url or slug
                        let href = c.get("url").and_then(|v| v.as_str()).map(|s| s.to_string())
                            .or_else(|| c.get("slug").and_then(|v| v.as_str()).map(|s| format!("{}/chapter/{}", BASE_URL, s)));
                        if let Some(u) = href {
                            chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: if title.is_empty() { u.clone() } else { title.clone() }, url: u, scraped: false });
                        }
                    }
                }
            }
        }
    }

    // Fallback: scan anchors
    if chapters.is_empty() {
        if let Ok(a_sel) = Selector::parse("a") {
            for a in document.select(&a_sel) {
                if let Some(href) = a.value().attr("href") {
                    if href.contains("/read/") || href.contains("/chapter/") {
                        let chapter_title = a.text().collect::<String>().trim().to_string();
                        chapters.push(Chapter {
                            id: 0,
                            manga_source_data_id: 0,
                            chapter_number: if chapter_title.is_empty() { href.to_string() } else { chapter_title },
                            url: if href.starts_with("http") { href.to_string() } else { format!("{}{}", BASE_URL, href) },
                            scraped: false,
                        });
                    }
                }
            }
        }
    }

    Ok(chapters)
}

pub async fn search_all_series_with_urls(client: &Client) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let mut page = 1u32;
    let mut out = Vec::new();
    loop {
        // Try search and series listing
        let urls = vec![
            format!("{}/search?query=&page={}", BASE_URL, page),
            format!("{}/series?page={}", BASE_URL, page),
        ];
        let mut items_in_page = 0;
        for url in urls {
            let response = client
                .get(&url)
                .header("User-Agent", "rust_manga_scraper/0.1.0")
                .header("Cookie", "nsfw=true; consent=true")
                .send()
                .await?
                .text()
                .await?;
            let document = Html::parse_document(&response);
            let a_sel = Selector::parse("a").unwrap();
            for a in document.select(&a_sel) {
                if let Some(href) = a.value().attr("href") {
                    if href.starts_with("/series/") {
                        let title_text = a.text().collect::<String>().trim().to_string();
                        let m = Manga { id: String::new(), title: if title_text.is_empty() { href.trim_start_matches("/series/").to_string() } else { title_text }, alt_titles: None, cover_url: None, description: None, tags: None, rating: None, monitored: None, check_interval_secs: None, discover_interval_secs: None, last_chapter_check: None, last_discover_check: None };
                        out.push((m, format!("{}{}", BASE_URL, href)));
                        items_in_page += 1;
                    }
                }
            }
            // Try Next.js json if present
            if items_in_page == 0 {
                if let Ok(script_sel) = Selector::parse("script#__NEXT_DATA__") {
                    if let Some(script) = document.select(&script_sel).next() {
                        let txt = script.text().collect::<String>();
                        if let Ok(json) = serde_json::from_str::<Value>(&txt) {
                            if let Some(arr) = json.pointer("/props/pageProps/series").and_then(|v| v.as_array()) {
                                for item in arr {
                                    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let slug = item.get("slug").and_then(|v| v.as_str()).unwrap_or("");
                                    if !slug.is_empty() {
                                        let series_url = format!("{}/series/{}", BASE_URL, slug);
                                        out.push((Manga { id: String::new(), title, alt_titles: None, cover_url: None, description: None, tags: None, rating: None, monitored: None, check_interval_secs: None, discover_interval_secs: None, last_chapter_check: None, last_discover_check: None }, series_url));
                                        items_in_page += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        if items_in_page == 0 { break; }
        page += 1;
        if page > 50 { break; }
    }

    // Fallback: parse sitemap for series links
    if out.is_empty() {
        let sitemap = client.get(format!("{}/sitemap.xml", BASE_URL)).header("User-Agent", "rust_manga_scraper/0.1.0").send().await?.text().await?;
        let re = regex::Regex::new(r"<loc>\s*(?P<loc>https?://[^<]+/series/[^<]+)\s*</loc>").unwrap();
        for cap in re.captures_iter(&sitemap) {
            let loc = cap.name("loc").unwrap().as_str().to_string();
            let title = loc.split('/').last().unwrap_or("").replace('-', " ");
            out.push((Manga { id: String::new(), title, alt_titles: None, cover_url: None, description: None, tags: None, rating: None, monitored: None, check_interval_secs: None, discover_interval_secs: None, last_chapter_check: None, last_discover_check: None }, loc));
        }
    }

    Ok(out)
}
