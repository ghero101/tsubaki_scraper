use reqwest::Client;
use scraper::{Html, Selector};
use crate::models::{Manga, Chapter};
use regex::Regex;

const BASE_URL: &str = "https://templescan.net";

pub async fn search_manga_with_urls(client: &Client, title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    if !title.trim().is_empty() {
        let url = format!("{}/?s={}&post_type=wp-manga", BASE_URL, title);
        let response = client.get(&url).send().await?.text().await?;
        let document = Html::parse_document(&response);
        let selector = Selector::parse("div.page-item-detail").unwrap();
        let mut out = Vec::new();
        for element in document.select(&selector) {
            if let Some(title_element) = element.select(&Selector::parse("h3 > a").unwrap()).next() {
                let title = title_element.text().collect::<String>().trim().to_string();
                let series_url = title_element.value().attr("href").unwrap_or("").to_string();
                let cover_url = element
                    .select(&Selector::parse("img").unwrap())
                    .next()
                    .and_then(|e| e.value().attr("src").or_else(|| e.value().attr("data-src")))
                    .map(|s| s.to_string());
                if !series_url.is_empty() {
                    out.push((Manga { id: String::new(), title, alt_titles: None, cover_url, description: None, tags: None, rating: None, monitored: None, check_interval_secs: None, discover_interval_secs: None, last_chapter_check: None, last_discover_check: None }, series_url));
                }
            }
        }
        return Ok(out);
    }
    let mut page = 1u32;
    let mut out = Vec::new();
    loop {
        let url = format!("{}/manga/?page={}", BASE_URL, page);
        let response = client.get(&url).send().await?.text().await?;
        let document = Html::parse_document(&response);
        let selector = Selector::parse("div.page-item-detail").unwrap();
        let mut items = 0;
        for element in document.select(&selector) {
            if let Some(title_element) = element.select(&Selector::parse("h3 > a").unwrap()).next() {
                let title = title_element.text().collect::<String>().trim().to_string();
                let series_url = title_element.value().attr("href").unwrap_or("").to_string();
                let cover_url = element
                    .select(&Selector::parse("img").unwrap())
                    .next()
                    .and_then(|e| e.value().attr("src").or_else(|| e.value().attr("data-src")))
                    .map(|s| s.to_string());
                if !series_url.is_empty() {
                    items += 1;
                    out.push((Manga { id: String::new(), title, alt_titles: None, cover_url, description: None, tags: None, rating: None, monitored: None, check_interval_secs: None, discover_interval_secs: None, last_chapter_check: None, last_discover_check: None }, series_url));
                }
            }
        }
        if items == 0 || page > 100 { break; }
        page += 1;
    }
    Ok(out)
}

fn derive_chapter_label(text: &str, href: &str) -> String {
    let t = text.trim();
    if !t.is_empty() && t != "#" { return t.to_string(); }
    let lower = href.to_lowercase();
    if let Some(cap) = Regex::new(r"chapter[-/](\d+(?:\.\d+)?)").unwrap().captures(&lower) { return format!("Ch.{}", &cap[1]); }
    if let Some(cap) = Regex::new(r"vol(?:ume)?[-/](\d+)").unwrap().captures(&lower) { return format!("Vol.{}", &cap[1]); }
    href.to_string()
}

pub async fn get_chapters(client: &Client, manga_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    let response = client.get(manga_url).send().await?.text().await?;
    let document = Html::parse_document(&response);
    let selectors = [
        "li.wp-manga-chapter a",
        "ul.main.version-chap li a",
        "div.listing-chapters_wrap a",
    ];
    let mut chapters = Vec::new();

    'outer: for sel in &selectors {
        if let Ok(selector) = Selector::parse(sel) {
            for a in document.select(&selector) {
                let chapter_title = a.text().collect::<String>().trim().to_string();
                if let Some(href) = a.value().attr("href") {
                    let label = derive_chapter_label(&chapter_title, href);
                    chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: label, url: href.to_string(), scraped: false });
                }
            }
            if !chapters.is_empty() { break 'outer; }
        }
    }

    if chapters.is_empty() {
        // AJAX fallback
        let mut post_id: Option<String> = None;
        if let Ok(sel) = Selector::parse("div#manga-chapters-holder") {
            if let Some(div) = document.select(&sel).next() {
                if let Some(did) = div.value().attr("data-id") { post_id = Some(did.to_string()); }
            }
        }
        if post_id.is_none() {
            let script_sel = Selector::parse("script").unwrap();
            let re = Regex::new(r"manga_id\s*=\s*(\d+)").unwrap();
            for s in document.select(&script_sel) {
                let t = s.text().collect::<String>();
                if let Some(cap) = re.captures(&t) { post_id = Some(cap[1].to_string()); break; }
            }
        }
        if let Some(pid) = post_id {
            let origin = reqwest::Url::parse(manga_url).ok().and_then(|u| { Some(format!("{}://{}", u.scheme(), u.host_str()?)) }).unwrap_or_else(|| BASE_URL.to_string());
            let ajax = format!("{}/wp-admin/admin-ajax.php", origin);
            let resp = client
                .post(&ajax)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .form(&[("action", "manga_get_chapters"), ("manga", pid.as_str())])
                .send()
                .await?;
            let text = resp.text().await?;
            let html = Html::parse_fragment(&text);
            let a_sel = Selector::parse("a").unwrap();
            for a in html.select(&a_sel) {
                if let Some(href) = a.value().attr("href") {
                    let t = a.text().collect::<String>().trim().to_string();
                    let label = derive_chapter_label(&t, href);
                    chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: label, url: href.to_string(), scraped: false });
                }
            }
        }
    }

    Ok(chapters)
}