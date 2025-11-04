use reqwest::{Client, Url};
use scraper::{Html, Selector};
use crate::models::{Manga, Chapter};

const BASE_URL: &str = "https://rizzcomic.com";

pub async fn search_manga(client: &Client, title: &str) -> Result<Vec<Manga>, reqwest::Error> {
    let url = format!("{}/?s={}&post_type=wp-manga", BASE_URL, title);
    let response = client.get(&url).send().await?.text().await?;
    let document = Html::parse_document(&response);
    let selector = Selector::parse("div.page-item-detail").unwrap();
    let mut manga_list = Vec::new();

    for element in document.select(&selector) {
        let title_selector = Selector::parse("h3 > a").unwrap();
        let title_element = element.select(&title_selector).next().unwrap();
        let title = title_element.text().collect::<String>().trim().to_string();
        let url = title_element.value().attr("href").unwrap().to_string();

        let cover_selector = Selector::parse("img").unwrap();
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
        manga.alt_titles = Some(format!("::URL::{}", url));
        manga_list.push(manga);
    }

    Ok(manga_list)
}

/// Returns (Manga, manga_url)
pub async fn search_manga_with_urls(client: &Client, title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // If title provided, use search; else crawl listing pages for robustness
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
// Listing crawl: try multiple listing paths and selectors
    use std::collections::HashSet;
    let mut page = 1u32;
    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    loop {
        let candidates = vec![
            format!("{}/manga/?page={}", BASE_URL, page),
            format!("{}/manga/page/{}/", BASE_URL, page),
            format!("{}/series/?page={}", BASE_URL, page),
        ];
        let mut items_in_page = 0;
        for url in candidates {
            let response = client.get(&url).send().await?.text().await?;
            let document = Html::parse_document(&response);
            let selectors = vec![
                Selector::parse("div.page-item-detail").unwrap(),
                Selector::parse("div.manga-item").unwrap(),
                Selector::parse("div.bsx").unwrap(),
            ];
            for sel in &selectors {
                for element in document.select(sel) {
                    let a_sel = Selector::parse("h3 > a, a.item-title, a.series-title, a\n").unwrap();
                    if let Some(a) = element.select(&a_sel).next() {
                        let title = a.text().collect::<String>().trim().to_string();
                        let series_url = a.value().attr("href").unwrap_or("").to_string();
                        if series_url.is_empty() || seen.contains(&series_url) { continue; }
                        let cover_url = element
                            .select(&Selector::parse("img").unwrap())
                            .next()
                            .and_then(|e| e.value().attr("src").or_else(|| e.value().attr("data-src")))
                            .map(|s| s.to_string());
                        seen.insert(series_url.clone());
                        out.push((Manga { id: String::new(), title, alt_titles: None, cover_url, description: None, tags: None, rating: None, monitored: None, check_interval_secs: None, discover_interval_secs: None, last_chapter_check: None, last_discover_check: None }, series_url));
                        items_in_page += 1;
                    }
                }
                if items_in_page > 0 { break; }
            }
            if items_in_page > 0 { break; }
        }
        if items_in_page == 0 || page > 100 { break; }
        page += 1;
    }
    Ok(out)
}

/// First-page only for quick checks
pub async fn search_manga_first_page(client: &Client) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    use std::collections::HashSet;
    let mut out = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let candidates = vec![
        format!("{}/manga/?page=1", BASE_URL),
        format!("{}/manga/page/1/", BASE_URL),
        format!("{}/series/?page=1", BASE_URL),
    ];
    for url in candidates {
        let response = client.get(&url).send().await?.text().await?;
        let document = Html::parse_document(&response);
        let selectors = vec![
            Selector::parse("div.page-item-detail").unwrap(),
            Selector::parse("div.manga-item").unwrap(),
            Selector::parse("div.bsx").unwrap(),
        ];
        for sel in &selectors {
            for element in document.select(sel) {
                // Try multiple link patterns
                let a_sel = Selector::parse("h3 > a, a.item-title, a.series-title, a").unwrap();
                if let Some(a) = element.select(&a_sel).next() {
                    // Get title from title attribute or link text
                    let title = a.value().attr("title")
                        .map(|s| s.to_string())
                        .or_else(|| Some(a.text().collect::<String>().trim().to_string()))
                        .unwrap_or_default();
                    let series_url = a.value().attr("href").unwrap_or("").to_string();
                    if title.is_empty() || series_url.is_empty() || seen.contains(&series_url) { continue; }
                    let cover_url = element
                        .select(&Selector::parse("img").unwrap())
                        .next()
                        .and_then(|e| e.value().attr("src").or_else(|| e.value().attr("data-src")))
                        .map(|s| s.to_string());
                    seen.insert(series_url.clone());
                    out.push((Manga { id: String::new(), title, alt_titles: None, cover_url, description: None, tags: None, rating: None, monitored: None, check_interval_secs: None, discover_interval_secs: None, last_chapter_check: None, last_discover_check: None }, series_url));
                }
            }
            if !out.is_empty() { return Ok(out); }
        }
    }
    Ok(out)
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
        "li.wp-manga-chapter a",
        "ul.main.version-chap li a",
        "div.listing-chapters_wrap a",
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
                    let abs = if let Some(b) = &base { b.join(href).map(|u| u.to_string()).unwrap_or_else(|_| href.to_string()) } else { href.to_string() };
                    chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: chapter_title.clone(), url: abs, scraped: false });
                }
            }
            if !chapters.is_empty() { break 'outer; }
        }
    }

    if chapters.is_empty() {
        // AJAX fallback like WP-Manga
        let mut post_id: Option<String> = None;
        if let Ok(sel) = Selector::parse("div#manga-chapters-holder") {
            if let Some(div) = document.select(&sel).next() {
                if let Some(did) = div.value().attr("data-id") { post_id = Some(did.to_string()); }
            }
        }
        if post_id.is_none() {
            let script_sel = Selector::parse("script").unwrap();
            let re = regex::Regex::new(r"manga_id\s*=\s*(\d+)").unwrap();
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
            // Try admin-ajax
            let ajax = format!("{}/wp-admin/admin-ajax.php", origin);
            let resp = client
                .post(&ajax)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("X-Requested-With", "XMLHttpRequest")
                .form(&[("action", "manga_get_chapters"), ("manga", pid.as_str())])
                .send()
                .await?;
            let text = resp.text().await?;
            let mut found_any = false;
            {
                let html = Html::parse_fragment(&text);
                let a_sel = Selector::parse("a").unwrap();
                for a in html.select(&a_sel) {
                    if let Some(href) = a.value().attr("href") {
                        let t = a.text().collect::<String>().trim().to_string();
                        chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: t, url: href.to_string(), scraped: false });
                        found_any = true;
                    }
                }
            }
            if !found_any {
                // Try common Madara AJAX path variants
                let variants = vec![
                    format!("{}/ajax/chapters?postId={}", origin, pid),
                    format!("{}/ajax/chapters/?postId={}", origin, pid),
                    format!("{}/ajax/chapters?id={}", origin, pid),
                ];
                for u in variants {
                    let res = client.get(&u).header("X-Requested-With", "XMLHttpRequest").send().await;
                    if let Ok(ok) = res {
                        if ok.status().is_success() {
                            if let Ok(body) = ok.text().await {
                                let frag = Html::parse_fragment(&body);
                                let a_sel = Selector::parse("a").unwrap();
                                let mut got = false;
                                for a in frag.select(&a_sel) {
                                    if let Some(href) = a.value().attr("href") {
                                        let t = a.text().collect::<String>().trim().to_string();
                                        chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: t, url: href.to_string(), scraped: false });
                                        got = true;
                                    }
                                }
                                if got { break; }
                            }
                        }
                    }
                }
            }
        }
        // Final fallbacks: scan anchors and pagination of series page
        if chapters.is_empty() {
            if let Ok(a_sel) = Selector::parse("a") {
                for a in document.select(&a_sel) {
                    if let Some(href) = a.value().attr("href") {
                        if href.contains("/chapter/") || href.contains("/read/") {
                            let t = a.text().collect::<String>().trim().to_string();
                            chapters.push(Chapter { id: 0, manga_source_data_id: 0, chapter_number: t, url: href.to_string(), scraped: false });
                        }
                    }
                }
            }
            // Try a couple of pagination pages
            for n in 2..=5u32 {
                let urls = vec![format!("{}?page={}", manga_url, n), format!("{}/page/{}/", manga_url.trim_end_matches('/'), n)];
                let mut any = false;
                for u in urls {
                    if let Ok(res) = client.get(&u).send().await { if let Ok(txt) = res.text().await {
                        let doc = Html::parse_document(&txt);
                        if let Ok(sel) = Selector::parse("a[href*='/chapter/'], li.wp-manga-chapter a, ul.main.version-chap li a") {
                            for a in doc.select(&sel) {
                                if let Some(href) = a.value().attr("href") {
                                    let t = a.text().collect::<String>().trim().to_string();
                                    chapters.push(Chapter { id:0, manga_source_data_id:0, chapter_number: t, url: href.to_string(), scraped:false });
                                    any = true;
                                }
                            }
                        }
                    }}
                }
                if !any { break; }
            }
        }
    }

    Ok(chapters)
}
