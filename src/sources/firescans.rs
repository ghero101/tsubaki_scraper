use reqwest::{Client, Url};
use scraper::{Html, Selector};
use crate::models::{Manga, Chapter};
use regex::Regex;

const BASE_URL: &str = "https://firescans.xyz";

pub async fn search_manga(client: &Client, title: &str) -> Result<Vec<Manga>, reqwest::Error> {
    let url = format!("{}/series?search={}", BASE_URL, title);
    let response = client.get(&url).send().await?.text().await?;
    let document = Html::parse_document(&response);
    let selector = Selector::parse("div.series-card").unwrap();
    let mut manga_list = Vec::new();

    for element in document.select(&selector) {
        let title_selector = Selector::parse("a.series-title").unwrap();
        let title_element = element.select(&title_selector).next().unwrap();
        let title = title_element.text().collect::<String>().trim().to_string();

        let url_selector = Selector::parse("a.series-title").unwrap();
        let url_element = element.select(&url_selector).next().unwrap();
        let url = url_element.value().attr("href").unwrap().to_string();

        let cover_selector = Selector::parse("img.series-poster").unwrap();
        let cover_element = element.select(&cover_selector).next().unwrap();
        let cover_url = cover_element.value().attr("src").unwrap().to_string();

        let mut manga = Manga {
            id: String::new(),
            title,
            alt_titles: None,
            cover_url: Some(cover_url),
            description: None,
            tags: None,
            rating: None,
            monitored: None,
            check_interval_secs: None,
            discover_interval_secs: None,
            last_chapter_check: None,
            last_discover_check: None,
        };
        let encoded = format!("::URL::{}", url);
        manga.alt_titles = Some(encoded);
        manga_list.push(manga);
    }

    Ok(manga_list)
}

/// Returns (Manga, manga_url)
pub async fn search_manga_with_urls(client: &Client, title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    if !title.trim().is_empty() {
        let list = search_manga(client, title).await?;
        let mut out = Vec::new();
        for mut m in list {
            let url = m.alt_titles.clone().unwrap_or_default().strip_prefix("::URL::").unwrap_or("").to_string();
            m.alt_titles = None;
            out.push((m, url));
        }
        return Ok(out);
    }
    // Crawl listing pages: try /manga/?page=n first, then fallback to /series
    let mut out = Vec::new();
    let mut page = 1u32;
    loop {
        let url = format!("{}/manga/?page={}", BASE_URL, page);
        let response = client.get(&url).send().await?.text().await?;
        let document = Html::parse_document(&response);
        let mut items = 0;
        // firescans cards
        for element in document.select(&Selector::parse("div.series-card").unwrap()) {
            if let Some(title_element) = element.select(&Selector::parse("a.series-title").unwrap()).next() {
                let title = title_element.text().collect::<String>().trim().to_string();
                let series_url = title_element.value().attr("href").unwrap_or("").to_string();
                let cover_url = element
                    .select(&Selector::parse("img.series-poster").unwrap())
                    .next()
                    .and_then(|e| e.value().attr("src").or_else(|| e.value().attr("data-src")))
                    .map(|s| s.to_string());
                if !series_url.is_empty() { items += 1; out.push((Manga { id: String::new(), title, alt_titles: None, cover_url, description: None, tags: None, rating: None, monitored: None, check_interval_secs: None, discover_interval_secs: None, last_chapter_check: None, last_discover_check: None }, series_url)); }
            }
        }
        if items == 0 { break; }
        page += 1;
        if page > 50 { break; }
    }
    if out.is_empty() {
        // fallback to /series for old-style listings
        let mut page2 = 1u32;
        loop {
            let url = format!("{}/series?page={}", BASE_URL, page2);
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
                    if !series_url.is_empty() { items += 1; out.push((Manga { id: String::new(), title, alt_titles: None, cover_url, description: None, tags: None, rating: None, monitored: None, check_interval_secs: None, discover_interval_secs: None, last_chapter_check: None, last_discover_check: None }, series_url)); }
                }
            }
            if items == 0 || page2 > 100 { break; }
            page2 += 1;
        }
    }
    Ok(out)
}

/// First-page only for quick checks
pub async fn search_manga_first_page(client: &Client) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let url = format!("{}/manga/?page=1", BASE_URL);
    let response = client.get(&url).send().await?.text().await?;
    let document = Html::parse_document(&response);
    let mut out = Vec::new();
    
    // Try multiple selectors for different FireScans layouts
    let selectors = vec![
        ("div.page-listing-item", "h3 a"),  // MadaraProject theme (current)
        ("div.series-card", "a.series-title"),  // Old layout
        ("div.page-item-detail", "h3 > a"),  // Standard WP-Manga
    ];
    
    for (container_sel, link_sel) in &selectors {
        if let Ok(container_selector) = Selector::parse(container_sel) {
            for element in document.select(&container_selector) {
                if let (Ok(link_selector), Some(title_element)) = (Selector::parse(link_sel), element.select(&Selector::parse(link_sel).unwrap()).next()) {
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
            if !out.is_empty() {
                break;  // Found manga with this selector, stop trying others
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

pub async fn get_chapters(client: &Client, manga_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    let response = client
        .get(manga_url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Referer", manga_url)
        .send()
        .await?
        .text()
        .await?;
    let document = Html::parse_document(&response);
    let selectors = [
        "ul.chapter-list li a",
        "div.chapter-list a",
        "div#chapters a",
        "li.wp-manga-chapter a",
        "a.chapter, a[href*='/chapter/']",
        "div.eplister a",
        "div#chapterlist a",
        "div.bxcl a",
    ];
    let mut chapters = Vec::new();
    let base = Url::parse(manga_url).ok();

    'outer: for sel in &selectors {
        if let Ok(selector) = Selector::parse(sel) {
            for a in document.select(&selector) {
                let chapter_title = a.text().collect::<String>().trim().to_string();
                if let Some(href) = a.value().attr("href").or_else(|| a.value().attr("data-href")) {
                    let label = derive_chapter_label(&chapter_title, href);
                    let abs = if let Some(b) = &base { b.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string()) } else { href.to_string() };
                    chapters.push(Chapter {
                        id: 0,
                        manga_source_data_id: 0,
                        chapter_number: label,
                        url: abs,
                        scraped: false,
                    });
                }
            }
            if !chapters.is_empty() { break 'outer; }
        }
    }

    // Final fallback: scan all anchors for likely chapter links
    if chapters.is_empty() {
        if let Ok(a_sel) = Selector::parse("a") {
            for a in document.select(&a_sel) {
                if let Some(href) = a.value().attr("href") {
                    if href.contains("/chapter/") || href.contains("/read/") {
                        let title = a.text().collect::<String>().trim().to_string();
                        let label = derive_chapter_label(&title, href);
                        chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: label, url: href.to_string(), scraped: false });
                    }
                }
            }
        }
    }

    if chapters.is_empty() {
        // Try paginated chapter lists
        for n in 2..=10u32 {
            let urls = vec![format!("{}/page/{}/", manga_url.trim_end_matches('/'), n), format!("{}?page={}", manga_url, n)];
            let mut any = false;
            for u in urls {
                let res = client.get(&u).send().await?.text().await?;
                let doc = Html::parse_document(&res);
                if let Ok(selector) = Selector::parse("a.chapter, ul.chapter-list li a, li.wp-manga-chapter a") {
                    for a in doc.select(&selector) {
                        if let Some(href) = a.value().attr("href") {
                            let title = a.text().collect::<String>().trim().to_string();
                            let label = derive_chapter_label(&title, href);
                            chapters.push(Chapter { id:0, manga_source_data_id:0, chapter_number: label, url: href.to_string(), scraped:false });
                            any = true;
                        }
                    }
                }
            }
            if !any { break; }
        }
    }

    if chapters.is_empty() {
        // Try WP-Manga AJAX for some FireScans deployments
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
            // derive origin from manga_url
            let origin = Url::parse(manga_url).ok().and_then(|u| {
                Some(format!("{}://{}", u.scheme(), u.host_str()?))
            }).unwrap_or_else(|| BASE_URL.to_string());
            // Try admin-ajax POST first
            let ajax = format!("{}/wp-admin/admin-ajax.php", origin);
            let resp = client
                .post(&ajax)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Origin", &origin)
                .header("Referer", manga_url)
                .form(&[("action", "manga_get_chapters"), ("manga", pid.as_str())])
                .send()
                .await?;
            let text = resp.text().await?;
            let html = Html::parse_fragment(&text);
            let a_sel = Selector::parse("a").unwrap();
            for a in html.select(&a_sel) {
                if let Some(href) = a.value().attr("href").or_else(|| a.value().attr("data-href")) {
                    let title = a.text().collect::<String>().trim().to_string();
                    let label = derive_chapter_label(&title, href);
                    let abs = Url::parse(&origin).ok().and_then(|b| b.join(href).ok()).map(|u| u.to_string()).unwrap_or_else(|| href.to_string());
                    chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: label, url: abs, scraped: false });
                }
            }
            // If still empty, try GET ajax variants
            if chapters.is_empty() {
                let variants = vec![
                    format!("{}/ajax/chapters?postId={}", origin, pid),
                    format!("{}/ajax/chapters/?postId={}", origin, pid),
                    format!("{}/ajax/chapters?id={}", origin, pid),
                ];
                'v: for u in variants {
                    if let Ok(ok) = client.get(&u).header("X-Requested-With", "XMLHttpRequest").send().await {
                        if ok.status().is_success() {
                            if let Ok(body) = ok.text().await {
                                let frag = Html::parse_fragment(&body);
                                for a in frag.select(&a_sel) {
                                    if let Some(href) = a.value().attr("href").or_else(|| a.value().attr("data-href")) {
                                        let t = a.text().collect::<String>().trim().to_string();
                                        let lbl = derive_chapter_label(&t, href);
                                        let abs = Url::parse(&origin).ok().and_then(|b| b.join(href).ok()).map(|u| u.to_string()).unwrap_or_else(|| href.to_string());
                                        chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: lbl, url: abs, scraped: false });
                                    }
                                }
                                if !chapters.is_empty() { break 'v; }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(chapters)
}
