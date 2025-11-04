use reqwest::{Client, Url};
use scraper::{Html, Selector};
use crate::models::{Manga, Chapter};
use regex::Regex;
use tokio::time::{sleep, Duration};

async fn fetch_text(client: &Client, url: &str) -> Result<String, reqwest::Error> {
    let mut last_err: Option<reqwest::Error> = None;
    for _ in 0..3 {
        match client.get(url).send().await {
            Ok(resp) => {
                match resp.error_for_status() {
                    Ok(ok) => { return ok.text().await; }
                    Err(e) => { last_err = Some(e); }
                }
            }
            Err(e) => { last_err = Some(e); }
        }
        sleep(Duration::from_millis(200)).await;
    }
    Err(last_err.unwrap())
}

pub async fn search_manga_with_urls_base(client: &Client, base_url: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // Try multiple URL patterns to support different site configurations
    let url_patterns = vec![
        "/manga/?page={}",   // Standard WP-Manga
        "/series?page={}",   // Alternative (stonescape, etc)
        "/?page={}",         // Home page pagination (some sites)
    ];
    
    let mut working_pattern: Option<&str> = None;
    let mut out = Vec::new();
    
    // Test first page with each pattern to find working one
    for pattern in &url_patterns {
        let test_url = base_url.to_owned() + &pattern.replace("{}", "1");
        if let Ok(response) = fetch_text(client, &test_url).await {
            let document = Html::parse_document(&response);
            let selector = Selector::parse("div.page-item-detail").unwrap();
            if document.select(&selector).next().is_some() {
                working_pattern = Some(pattern);
                break;
            }
        }
    }
    
    let pattern = working_pattern.unwrap_or("/manga/?page={}");
    let mut page = 1u32;
    
    loop {
        let url = base_url.to_owned() + &pattern.replace("{}", &page.to_string());
        let response = fetch_text(client, &url).await?;
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
        sleep(Duration::from_millis(150)).await;
    }
    Ok(out)
}

pub async fn search_manga_first_page(client: &Client, base_url: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // Try multiple URL patterns
    let url_patterns = vec![
        "/manga/?page=1",
        "/series?page=1",
        "/?page=1",
        "/",  // Just root for some sites
    ];
    
    let mut response_text = String::new();
    for pattern in &url_patterns {
        let url = base_url.to_owned() + pattern;
        if let Ok(text) = fetch_text(client, &url).await {
            let doc = Html::parse_document(&text);
            let selector = Selector::parse("div.page-item-detail").unwrap();
            if doc.select(&selector).next().is_some() {
                response_text = text;
                break;
            }
        }
    }
    
    if response_text.is_empty() {
        // Fallback to default if none worked
        response_text = fetch_text(client, &format!("{}/manga/?page=1", base_url)).await?;
    }
    
    let document = Html::parse_document(&response_text);
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

pub async fn get_chapters_base(client: &Client, base_url: &str, series_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    let response = fetch_text(client, series_url).await?;
    let document = Html::parse_document(&response);
    let selectors = [
        "li.wp-manga-chapter a",
        "ul.main.version-chap li a",
        "div.listing-chapters_wrap a",
        "div.eplister a",
        "div.bxcl a",
        "div#chapterlist a",
    ];
    let mut chapters = Vec::new();
    let series_base = Url::parse(series_url).ok();

    'outer: for sel in &selectors {
        if let Ok(selector) = Selector::parse(sel) {
            for a in document.select(&selector) {
                let chapter_title = a.text().collect::<String>().trim().to_string();
                if let Some(href) = a.value().attr("href").or_else(|| a.value().attr("data-href")) {
                    let label = derive_chapter_label(&chapter_title, href);
                    let abs = if let Some(base) = &series_base { base.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string()) } else { href.to_string() };
                    chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: label, url: abs, scraped: false });
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
            let ajax = format!("{}/wp-admin/admin-ajax.php", base_url);
            // POST not retried; single attempt
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
                if let Some(href) = a.value().attr("href").or_else(|| a.value().attr("data-href")) {
                    let t = a.text().collect::<String>().trim().to_string();
                    let label = derive_chapter_label(&t, href);
                    let abs = if let Some(base) = &series_base { base.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string()) } else { href.to_string() };
                    chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: label, url: abs, scraped: false });
                }
            }
        }
    }

    Ok(chapters)
}