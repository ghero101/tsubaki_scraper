use crate::models::{Chapter, Manga};
use regex::Regex;
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use tokio::time::{sleep, Duration};

/// Clean manga title by removing common metadata, badges, and navigation elements
pub fn clean_manga_title_public(title: &str) -> Option<String> {
    clean_manga_title(title)
}

fn clean_manga_title(title: &str) -> Option<String> {
    let mut cleaned = title.to_string();

    // Normalize whitespace first
    cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");

    // Remove common prefixes
    let prefixes = [
        "MANHWA",
        "MANHUA",
        "MANGA",
        "Manhua",
        "Manga",
        "cover art not final",
        "Read free!",
        "Comics",
        "🔥 Hot",
        "Hot",
        "NEW",
        "New",
    ];
    for prefix in &prefixes {
        if cleaned.to_lowercase().starts_with(&prefix.to_lowercase()) {
            cleaned = cleaned[prefix.len()..].trim().to_string();
        }
    }

    // Remove ratings at end (e.g., "9.3", "9.98", "10")
    let rating_re = Regex::new(r"\s*\d+\.?\d*\s*$").unwrap();
    cleaned = rating_re.replace(&cleaned, "").to_string();

    // Remove "Chapter X" patterns at end
    let chapter_re = Regex::new(r"\s*Chapter\s+\d+\.?\d*\s*$").unwrap();
    cleaned = chapter_re.replace(&cleaned, "").to_string();

    // Remove genre/category tags at end (Drama, Action, Romance, etc.)
    let genre_re = Regex::new(r"\s*(Drama|Action|Romance|Fantasy|Comedy|Horror|Thriller|Mystery|Shoujo|Shounen|Seinen|Josei|Webtoon)\s*$").unwrap();
    cleaned = genre_re.replace(&cleaned, "").to_string();

    // Remove "Start reading" and similar CTAs
    let cta_re = Regex::new(r"(Start [Rr]eading.*|Read Now.*|Add to Library.*)$").unwrap();
    cleaned = cta_re.replace(&cleaned, "").to_string();

    // Remove rating indicators
    let rating_text_re = Regex::new(r"\d+Rating\d*Chapters?").unwrap();
    cleaned = rating_text_re.replace(&cleaned, "").to_string();

    // Normalize whitespace again after removals
    cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    cleaned = cleaned.trim().to_string();

    // Skip if empty or too short
    if cleaned.is_empty() || cleaned.len() < 2 {
        return None;
    }

    // Skip if only special characters or numbers
    if cleaned.chars().all(|c| !c.is_alphanumeric()) {
        return None;
    }

    // Skip single/double letter codes (KR, EN, JP, etc.)
    if cleaned.len() <= 2 && cleaned.chars().all(|c| c.is_ascii_uppercase()) {
        return None;
    }

    // Skip short numeric-only titles (likely page numbers or navigation)
    // Allow longer numeric titles like "86" (2 chars) but filter out very short ones
    if cleaned.len() <= 3 && cleaned.chars().all(|c| c.is_numeric()) {
        return None;
    }

    // Skip common navigation/UI elements (English and Korean)
    let skip_list = [
        "next", "prev", "previous", "home", "menu", "search", "login", "register", "series",
        "action", "manga", "older", "upcoming", "novel", "comics",
        "랭킹", // Korean: "Ranking"
        "요일", // Korean: "Day of week"
        "신작", // Korean: "New releases"
        "장르", // Korean: "Genre"
        "완결", // Korean: "Completed"
    ];
    // Check both lowercase English and original (for non-ASCII like Korean)
    if skip_list.contains(&cleaned.to_lowercase().as_str()) || skip_list.contains(&cleaned.as_str())
    {
        return None;
    }

    // Skip if starts with "Chapter" (likely a chapter link, not a title)
    if cleaned.to_lowercase().starts_with("chapter ") {
        return None;
    }

    Some(cleaned)
}

async fn fetch_text(client: &Client, url: &str) -> Result<String, reqwest::Error> {
    let mut last_err: Option<reqwest::Error> = None;
    let retry_delays = [500, 1000, 2000, 4000]; // Exponential backoff in milliseconds

    for (attempt, &delay) in retry_delays.iter().enumerate() {
        // Enhanced headers to bypass bot detection
        let request = client.get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Cache-Control", "max-age=0");

        match request.send().await {
            Ok(resp) => {
                let status = resp.status();
                // Retry on rate limiting, server errors, and Cloudflare errors
                let is_retryable = matches!(
                    status.as_u16(),
                    429 | 500 | 502 | 503 | 504 | 520 | 521 | 522 | 523 | 524
                );

                if is_retryable && attempt < retry_delays.len() - 1 {
                    log::warn!(
                        "Retryable status {} for {}, retrying in {}ms",
                        status,
                        url,
                        delay
                    );
                    sleep(Duration::from_millis(delay)).await;
                    continue;
                }
                match resp.error_for_status() {
                    Ok(ok) => {
                        return ok.text().await;
                    }
                    Err(e) => {
                        last_err = Some(e);
                    }
                }
            }
            Err(e) => {
                // Retry on network errors
                if (e.is_timeout() || e.is_connect() || e.is_request())
                    && attempt < retry_delays.len() - 1
                {
                    log::warn!("Network error for {}, retrying in {}ms: {}", url, delay, e);
                    sleep(Duration::from_millis(delay)).await;
                    last_err = Some(e);
                    continue;
                }
                last_err = Some(e);
            }
        }
        if attempt < retry_delays.len() - 1 {
            sleep(Duration::from_millis(delay)).await;
        }
    }
    Err(last_err.unwrap())
}

pub async fn search_manga_with_urls_base(
    client: &Client,
    base_url: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // Try multiple URL patterns to support different site configurations
    let url_patterns = vec![
        "/manga/?page={}", // Standard WP-Manga
        "/series?page={}", // Alternative (stonescape, etc)
        "/?page={}",       // Home page pagination (some sites)
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

        // Try multiple selector patterns for different theme types
        // Order matters: try most specific first
        let selector_patterns = vec![
            ("div.page-item-detail", "h3 > a"),   // Standard WP-Manga
            ("div.page-listing-item", "h3 a"),    // MadaraProject theme (firescans, etc)
            ("div.listupd .bs .bsx", "a"),        // MangaStream nested (rizzcomic)
            ("div.bsx", "a"),                     // MangaStream/MangaBuddy theme
            ("div.manga-item", "a.manga-link"),   // Custom theme
            ("div.utao .uta .imgu", "a"),         // MangaStream variant
            ("article.bs", "a"),                  // Article-based layout
            ("div.post-item", "h2 a"),            // Post-based layout
            ("div.series-item", "a.series-link"), // Series layout
        ];

        let mut items = 0;

        for (container_sel, link_sel) in &selector_patterns {
            if let Ok(container_selector) = Selector::parse(container_sel) {
                for element in document.select(&container_selector) {
                    let mut title: String;
                    let series_url: String;

                    if let Some(link_element) =
                        element.select(&Selector::parse(link_sel).unwrap()).next()
                    {
                        series_url = link_element.value().attr("href").unwrap_or("").to_string();

                        // Try multiple ways to get title
                        if *link_sel == "h3 > a" || *link_sel == "h3 a" || *link_sel == "h2 a" {
                            title = link_element.text().collect::<String>().trim().to_string();
                        } else {
                            // For other patterns, try title attribute first, then text
                            title = link_element
                                .value()
                                .attr("title")
                                .map(|s| s.to_string())
                                .or_else(|| {
                                    Some(link_element.text().collect::<String>().trim().to_string())
                                })
                                .unwrap_or_default();
                        }

                        // Apply comprehensive title cleaning
                        title = match clean_manga_title(&title) {
                            Some(cleaned) => cleaned,
                            None => continue, // Skip if cleaning filtered it out
                        };

                        let cover_url = element
                            .select(&Selector::parse("img").unwrap())
                            .next()
                            .and_then(|e| {
                                e.value().attr("src").or_else(|| e.value().attr("data-src"))
                            })
                            .map(|s| s.to_string());

                        if !series_url.is_empty() && !title.is_empty() {
                            items += 1;
                            out.push((
                                Manga {
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
                                },
                                series_url,
                            ));
                        }
                    }
                }

                // If we found items with this pattern, stop trying others
                if items > 0 {
                    break;
                }
            }
        }
        if items == 0 || page > 100 {
            break;
        }
        page += 1;
        sleep(Duration::from_millis(150)).await;
    }
    Ok(out)
}

pub async fn search_manga_first_page(
    client: &Client,
    base_url: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // Try multiple URL patterns and parse each until we find results
    let url_patterns = vec![
        "/", // Many sites list on root
        "/?page=1",
        "/manga/?page=1",
        "/series?page=1",
    ];

    // Selector patterns to extract items from a fetched page
    let selector_patterns = vec![
        ("div.page-item-detail", "h3 > a"),   // Standard WP-Manga
        ("div.page-listing-item", "h3 a"),    // MadaraProject theme
        ("div.listupd .bs .bsx", "a"),        // MangaStream nested
        ("div.bsx", "a"),                     // MangaStream/MangaBuddy theme
        ("div.manga-item", "a.manga-link"),   // Custom theme
        ("div.utao .uta .imgu", "a"),         // MangaStream variant
        ("article.bs", "a"),                  // Article-based layout
        ("div.post-item", "h2 a"),            // Post-based layout
        ("div.series-item", "a.series-link"), // Series layout
    ];

    for pattern in &url_patterns {
        let url = base_url.to_owned() + pattern;
        if let Ok(text) = fetch_text(client, &url).await {
            let document = Html::parse_document(&text);
            let mut out = Vec::new();
            for (container_sel, link_sel) in &selector_patterns {
                if let Ok(container_selector) = Selector::parse(container_sel) {
                    for element in document.select(&container_selector) {
                        if let Some(link_element) =
                            element.select(&Selector::parse(link_sel).unwrap()).next()
                        {
                            let series_url =
                                link_element.value().attr("href").unwrap_or("").to_string();
                            let mut title: String = if *link_sel == "h3 > a"
                                || *link_sel == "h3 a"
                                || *link_sel == "h2 a"
                            {
                                link_element.text().collect::<String>().trim().to_string()
                            } else {
                                link_element
                                    .value()
                                    .attr("title")
                                    .map(|s| s.to_string())
                                    .or_else(|| {
                                        Some(
                                            link_element
                                                .text()
                                                .collect::<String>()
                                                .trim()
                                                .to_string(),
                                        )
                                    })
                                    .unwrap_or_default()
                            };

                            // Apply comprehensive title cleaning
                            title = match clean_manga_title(&title) {
                                Some(cleaned) => cleaned,
                                None => continue, // Skip if cleaning filtered it out
                            };

                            let cover_url = element
                                .select(&Selector::parse("img").unwrap())
                                .next()
                                .and_then(|e| {
                                    e.value().attr("src").or_else(|| e.value().attr("data-src"))
                                })
                                .map(|s| s.to_string());
                            if !series_url.is_empty() && !title.is_empty() {
                                out.push((
                                    Manga {
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
                                    },
                                    series_url,
                                ));
                            }
                        }
                    }
                    if !out.is_empty() {
                        return Ok(out);
                    }
                }
            }
            // Fallback: scan all anchors for likely series links if structured selectors failed
            use std::collections::HashSet;
            let mut seen: HashSet<String> = HashSet::new();
            let a_sel = Selector::parse("a").unwrap();
            for a in document.select(&a_sel) {
                if let Some(href) = a.value().attr("href") {
                    let h = href.trim();
                    if h.is_empty() {
                        continue;
                    }
                    // Heuristics: include links that look like series pages, exclude chapters/tags
                    let l = h.to_lowercase();
                    let looks_series = l.contains("/series/")
                        || (l.contains("/manga/") && !l.contains("/chapter/"));
                    if !looks_series {
                        continue;
                    }
                    let title_text_raw = a
                        .value()
                        .attr("title")
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .unwrap_or_else(|| a.text().collect::<String>().trim().to_string());

                    // Apply comprehensive title cleaning
                    let title_text = match clean_manga_title(&title_text_raw) {
                        Some(cleaned) => cleaned,
                        None => continue, // Skip if cleaning filtered it out
                    };

                    let series_url = if h.starts_with("http") {
                        h.to_string()
                    } else {
                        let path = if h.starts_with('/') {
                            h.to_string()
                        } else {
                            format!("/{}", h)
                        };
                        format!("{}{}", base_url.trim_end_matches('/'), path)
                    };
                    // title_text is already cleaned and validated by clean_manga_title
                    if seen.insert(series_url.clone()) {
                        out.push((
                            Manga {
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
                            },
                            series_url,
                        ));
                    }
                }
            }
            if !out.is_empty() {
                return Ok(out);
            }
            // If no selectors or fallbacks matched on this pattern, try next pattern
        }
    }
    // Nothing found across patterns
    Ok(Vec::new())
}

fn derive_chapter_label(text: &str, href: &str) -> String {
    let t = text.trim();
    if !t.is_empty() && t != "#" {
        return t.to_string();
    }
    let lower = href.to_lowercase();
    if let Some(cap) = Regex::new(r"chapter[-/](\d+(?:\.\d+)?)")
        .unwrap()
        .captures(&lower)
    {
        return format!("Ch.{}", &cap[1]);
    }
    if let Some(cap) = Regex::new(r"vol(?:ume)?[-/](\d+)")
        .unwrap()
        .captures(&lower)
    {
        return format!("Vol.{}", &cap[1]);
    }
    href.to_string()
}

pub async fn get_chapters_base(
    client: &Client,
    base_url: &str,
    series_url: &str,
) -> Result<Vec<Chapter>, reqwest::Error> {
    let response = fetch_text(client, series_url).await?;
    let document = Html::parse_document(&response);
    let selectors = [
        "li.wp-manga-chapter a",
        "ul.main.version-chap li a",
        "div.listing-chapters_wrap a",
        "div.eplister a",
        "div.bxcl a",
        "div#chapterlist a",
        "div.chapter-list a",
        "ul.chapter-list a",
        "li.chapter a",
        "div.chapters-list a",
        "ul.chapters a",
        // Additional common patterns for various WP-Manga themes
        "div.chbox a",                                 // Chapter box variant
        "ul.clstyle a",                                // Chapter list style
        "div.epcheck a",                               // Episode check variant
        "div.chapterlist a",                           // Single word variant (no hyphen)
        "ul.version-chap a",                           // Version chapter without ul.main
        "div.chapter-item a",                          // Chapter item variant
        "li.parent.has-child a",                       // Nested chapter lists
        "div.eplister ul li a",                        // Nested eplister
        "div#chapter-heading + div a",                 // Chapters after heading
        "div.page-content-listing a[href*='chapter']", // Content listing with chapter in URL
        "ul.list-chapters a",                          // List chapters variant
        "div.manga-chapters a[href*='chapter']",       // Manga chapters container
    ];
    let mut chapters = Vec::new();
    let series_base = Url::parse(series_url).ok();

    'outer: for sel in &selectors {
        if let Ok(selector) = Selector::parse(sel) {
            for a in document.select(&selector) {
                let chapter_title = a.text().collect::<String>().trim().to_string();
                if let Some(href) = a
                    .value()
                    .attr("href")
                    .or_else(|| a.value().attr("data-href"))
                {
                    let label = derive_chapter_label(&chapter_title, href);
                    let abs = if let Some(base) = &series_base {
                        base.join(href)
                            .map(|u| u.to_string())
                            .unwrap_or_else(|_| href.to_string())
                    } else {
                        href.to_string()
                    };
                    chapters.push(Chapter {
                        id: 0,
                        manga_source_data_id: 0,
                        chapter_number: label,
                        url: abs,
                        scraped: false,
                    });
                }
            }
            if !chapters.is_empty() {
                log::debug!("Found {} chapters using selector: {}", chapters.len(), sel);
                break 'outer;
            }
        }
    }

    // Fallback: look for "Read First/Last" buttons and generate chapter range
    if chapters.is_empty() {
        if let Ok(a_sel) = Selector::parse("a[id='btn-read-first'], a[id='btn-read-last']") {
            let mut first_num = 1u32;
            let mut last_num = 1u32;

            for a in document.select(&a_sel) {
                if let Some(href) = a.value().attr("href") {
                    // Extract chapter number from URL like ".../chapter-13/"
                    if let Some(cap) = Regex::new(r"chapter[-/](\d+)").unwrap().captures(href) {
                        if let Ok(num) = cap[1].parse::<u32>() {
                            if a.value().attr("id") == Some("btn-read-first") {
                                last_num = num;
                            } else {
                                first_num = num;
                            }
                        }
                    }
                }
            }

            // Generate all chapters in range
            if last_num >= first_num && last_num > 0 {
                let manga_base = series_url.trim_end_matches('/');
                for n in first_num..=last_num {
                    let chapter_url = format!("{}/chapter-{}/", manga_base, n);
                    chapters.push(Chapter {
                        id: 0,
                        manga_source_data_id: 0,
                        chapter_number: format!("Chapter {}", n),
                        url: chapter_url,
                        scraped: false,
                    });
                }
            }
        }
    }

    if chapters.is_empty() {
        // AJAX fallback
        let mut post_id: Option<String> = None;
        if let Ok(sel) = Selector::parse("div#manga-chapters-holder") {
            if let Some(div) = document.select(&sel).next() {
                if let Some(did) = div.value().attr("data-id") {
                    post_id = Some(did.to_string());
                }
            }
        }
        if post_id.is_none() {
            let script_sel = Selector::parse("script").unwrap();
            let re = Regex::new(r"manga_id\s*=\s*(\d+)").unwrap();
            for s in document.select(&script_sel) {
                let t = s.text().collect::<String>();
                if let Some(cap) = re.captures(&t) {
                    post_id = Some(cap[1].to_string());
                    break;
                }
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
                if let Some(href) = a
                    .value()
                    .attr("href")
                    .or_else(|| a.value().attr("data-href"))
                {
                    let t = a.text().collect::<String>().trim().to_string();
                    let label = derive_chapter_label(&t, href);
                    let abs = if let Some(base) = &series_base {
                        base.join(href)
                            .map(|u| u.to_string())
                            .unwrap_or_else(|_| href.to_string())
                    } else {
                        href.to_string()
                    };
                    chapters.push(Chapter {
                        id: 0,
                        manga_source_data_id: 0,
                        chapter_number: label,
                        url: abs,
                        scraped: false,
                    });
                }
            }
        }
    }

    // Final fallback: scan all anchors for chapter-like URLs
    if chapters.is_empty() {
        log::debug!(
            "All primary selectors failed, trying final anchor scan for: {}",
            series_url
        );
        if let Ok(a_sel) = Selector::parse("a") {
            let mut seen_urls = std::collections::HashSet::new();
            for a in document.select(&a_sel) {
                if let Some(href) = a.value().attr("href") {
                    let lower = href.to_lowercase();

                    // Look for chapter-like patterns in URLs
                    let is_chapter = lower.contains("/chapter")
                        || lower.contains("/read/")
                        || lower.contains("/episode/")
                        || lower.contains("-chapter-")
                        || lower.contains("/ch-")
                        || lower.contains("/chap-");

                    // Skip navigation and non-chapter links
                    let is_navigation = lower.contains("/page/")
                        || lower.contains("/category/")
                        || lower.contains("/tag/")
                        || lower.contains("/author/")
                        || lower.contains("/genre/")
                        || lower.contains("?s=")
                        || lower.contains("/search");

                    if is_chapter && !is_navigation {
                        let t = a.text().collect::<String>().trim().to_string();

                        // Skip if text looks like navigation
                        if let Some(cleaned) = clean_manga_title(&t) {
                            // If title cleaning returns something, it's likely not a chapter
                            // Chapters typically have numbers/dates that get filtered
                            if cleaned.len() > 10 {
                                continue;
                            }
                        }

                        let label = derive_chapter_label(&t, href);
                        let abs = if let Some(base) = &series_base {
                            base.join(href)
                                .map(|u| u.to_string())
                                .unwrap_or_else(|_| href.to_string())
                        } else {
                            href.to_string()
                        };

                        // Only add if it looks like a valid chapter URL and we haven't seen it
                        if !label.is_empty()
                            && abs.contains("http")
                            && seen_urls.insert(abs.clone())
                        {
                            chapters.push(Chapter {
                                id: 0,
                                manga_source_data_id: 0,
                                chapter_number: label,
                                url: abs,
                                scraped: false,
                            });
                        }
                    }
                }
            }
        }
        if !chapters.is_empty() {
            log::debug!("Final fallback found {} chapters", chapters.len());
        } else {
            log::warn!("No chapters found for {} after all fallbacks", series_url);
        }
    }

    Ok(chapters)
}
