/// Browser-based implementations for JavaScript-rendered manga sources
///
/// These implementations use headless Chrome to handle sites that require
/// JavaScript execution or have heavy bot detection/Cloudflare protection.
///
/// Usage: Enable browser support in config.toml by setting:
/// [bot_detection]
/// enable_browser = true

use crate::browser_client::BrowserClient;
use crate::models::{Manga, Chapter};
use scraper::{Html, Selector};

/// Asmotoon - Browser-based implementation
pub mod asmotoon_browser {
    use super::*;

    const BASE_URL: &str = "https://asmotoon.com";

    pub async fn search_manga_with_urls(title: &str) -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;
        let url = format!("{}/?s={}&post_type=wp-manga", BASE_URL, title);

        log::info!("Fetching manga list from {} using browser", url);
        let html = browser.get_html_wait_for(&url, "div.page-item-detail", Some(std::time::Duration::from_secs(10)))?;

        Ok(parse_search_page(&html))
    }

    pub async fn get_chapters(manga_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;

        log::info!("Fetching chapters from {} using browser", manga_url);
        let html = browser.get_html_wait_for(manga_url, "li.wp-manga-chapter", Some(std::time::Duration::from_secs(10)))?;

        let document = Html::parse_document(&html);
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
}

/// Generic browser-based WP-Manga scraper for JavaScript-heavy sites
pub mod wp_manga_browser {
    use super::*;

    pub async fn search_manga_with_urls_base(base_url: &str) -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;

        // Try multiple URL patterns
        let url_patterns = vec![
            "/manga/?page=1",
            "/series?page=1",
            "/?page=1",
        ];

        for pattern in &url_patterns {
            let url = base_url.to_owned() + pattern;
            log::info!("Trying URL pattern: {}", url);

            match browser.get_html_wait_for(&url, "div.page-item-detail, div.bsx, article.bs", Some(std::time::Duration::from_secs(10))) {
                Ok(html) => {
                    let results = parse_manga_page(&html, base_url);
                    if !results.is_empty() {
                        log::info!("Found {} manga using pattern {}", results.len(), pattern);
                        return Ok(results);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to fetch with pattern {}: {}", pattern, e);
                }
            }
        }

        Ok(Vec::new())
    }

    pub async fn get_chapters_base(base_url: &str, series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;

        log::info!("Fetching chapters from {} using browser", series_url);
        let html = browser.get_html_wait_for(series_url, "li.wp-manga-chapter, div.eplister, div.bxcl", Some(std::time::Duration::from_secs(10)))?;

        let document = Html::parse_document(&html);
        let selectors = [
            "li.wp-manga-chapter a",
            "ul.main.version-chap li a",
            "div.listing-chapters_wrap a",
            "div.eplister a",
            "div.bxcl a",
            "div#chapterlist a",
        ];

        let mut chapters = Vec::new();

        for sel in &selectors {
            if let Ok(selector) = Selector::parse(sel) {
                for a in document.select(&selector) {
                    let chapter_title = a.text().collect::<String>().trim().to_string();
                    if let Some(href) = a.value().attr("href") {
                        let url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            let path = if href.starts_with('/') {
                                href.to_string()
                            } else {
                                format!("/{}", href)
                            };
                            format!("{}{}", base_url.trim_end_matches('/'), path)
                        };

                        chapters.push(Chapter {
                            id: 0,
                            manga_source_data_id: 0,
                            chapter_number: chapter_title,
                            url,
                            scraped: false,
                        });
                    }
                }
                if !chapters.is_empty() {
                    break;
                }
            }
        }

        Ok(chapters)
    }

    pub fn parse_manga_page(html: &str, _base_url: &str) -> Vec<(Manga, String)> {
        let document = Html::parse_document(html);
        let selector_patterns = vec![
            ("div.page-item-detail", "h3 > a"),
            ("div.page-listing-item", "h3 a"),
            ("div.listupd .bs .bsx", "a"),
            ("div.bsx", "a"),
            ("article.bs", "a"),
        ];

        for (container_sel, link_sel) in &selector_patterns {
            if let Ok(container_selector) = Selector::parse(container_sel) {
                let mut results = Vec::new();

                for element in document.select(&container_selector) {
                    if let Some(link_element) = element.select(&Selector::parse(link_sel).unwrap()).next() {
                        let series_url = link_element.value().attr("href").unwrap_or("").to_string();
                        let title = if *link_sel == "h3 > a" || *link_sel == "h3 a" {
                            link_element.text().collect::<String>().trim().to_string()
                        } else {
                            link_element.value().attr("title")
                                .map(|s| s.to_string())
                                .or_else(|| Some(link_element.text().collect::<String>().trim().to_string()))
                                .unwrap_or_default()
                        };

                        let cover_url = element
                            .select(&Selector::parse("img").unwrap())
                            .next()
                            .and_then(|e| e.value().attr("src").or_else(|| e.value().attr("data-src")))
                            .map(|s| s.to_string());

                        if !series_url.is_empty() && !title.is_empty() {
                            results.push((
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

                if !results.is_empty() {
                    return results;
                }
            }
        }

        Vec::new()
    }
}

/// Hivetoons - Browser implementation
pub mod hivetoons_browser {
    use super::*;
    const BASE_URL: &str = "https://hivetoons.com";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::search_manga_with_urls_base(BASE_URL).await
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::get_chapters_base(BASE_URL, series_url).await
    }
}

/// KenScans - Browser implementation
pub mod kenscans_browser {
    use super::*;
    const BASE_URL: &str = "https://kenscans.com";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::search_manga_with_urls_base(BASE_URL).await
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::get_chapters_base(BASE_URL, series_url).await
    }
}

/// QiScans - Browser implementation
pub mod qiscans_browser {
    use super::*;
    const BASE_URL: &str = "https://qiscans.org";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::search_manga_with_urls_base(BASE_URL).await
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::get_chapters_base(BASE_URL, series_url).await
    }
}

/// NyxScans - Browser implementation
pub mod nyxscans_browser {
    use super::*;
    const BASE_URL: &str = "https://nyxscans.com";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::search_manga_with_urls_base(BASE_URL).await
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::get_chapters_base(BASE_URL, series_url).await
    }
}

/// DrakeComic - Browser implementation with Cloudflare bypass
pub mod drakecomic_browser {
    use super::*;
    const BASE_URL: &str = "https://drakecomic.org";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;
        let url = format!("{}/manga/?page=1", BASE_URL);

        log::info!("Fetching from {} with Cloudflare bypass", url);
        let html = browser.navigate_with_cloudflare_bypass(&url)?;

        Ok(super::wp_manga_browser::parse_manga_page(&html, BASE_URL))
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;

        log::info!("Fetching chapters from {} with Cloudflare bypass", series_url);
        let html = browser.navigate_with_cloudflare_bypass(series_url)?;

        // Parse chapters from the HTML directly
        let document = Html::parse_document(&html);
        let selectors = [
            "li.wp-manga-chapter a",
            "ul.main.version-chap li a",
            "div.listing-chapters_wrap a",
            "div.eplister a",
            "div.bxcl a",
            "div#chapterlist a",
        ];

        let mut chapters = Vec::new();

        for sel in &selectors {
            if let Ok(selector) = Selector::parse(sel) {
                for a in document.select(&selector) {
                    let chapter_title = a.text().collect::<String>().trim().to_string();
                    if let Some(href) = a.value().attr("href") {
                        let url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            let path = if href.starts_with('/') {
                                href.to_string()
                            } else {
                                format!("/{}", href)
                            };
                            format!("{}{}", BASE_URL.trim_end_matches('/'), path)
                        };

                        chapters.push(Chapter {
                            id: 0,
                            manga_source_data_id: 0,
                            chapter_number: chapter_title,
                            url,
                            scraped: false,
                        });
                    }
                }
                if !chapters.is_empty() {
                    break;
                }
            }
        }

        Ok(chapters)
    }
}

/// MadaraScans - Browser implementation with Cloudflare bypass
pub mod madarascans_browser {
    use super::*;
    const BASE_URL: &str = "https://madarascans.com";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;
        let url = format!("{}/manga/?page=1", BASE_URL);

        log::info!("Fetching from {} with Cloudflare bypass", url);
        let html = browser.navigate_with_cloudflare_bypass(&url)?;

        Ok(super::wp_manga_browser::parse_manga_page(&html, BASE_URL))
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;

        log::info!("Fetching chapters from {} with Cloudflare bypass", series_url);
        let html = browser.navigate_with_cloudflare_bypass(series_url)?;

        // Parse chapters from the HTML directly
        let document = Html::parse_document(&html);
        let selectors = [
            "li.wp-manga-chapter a",
            "ul.main.version-chap li a",
            "div.listing-chapters_wrap a",
            "div.eplister a",
            "div.bxcl a",
            "div#chapterlist a",
        ];

        let mut chapters = Vec::new();

        for sel in &selectors {
            if let Ok(selector) = Selector::parse(sel) {
                for a in document.select(&selector) {
                    let chapter_title = a.text().collect::<String>().trim().to_string();
                    if let Some(href) = a.value().attr("href") {
                        let url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            let path = if href.starts_with('/') {
                                href.to_string()
                            } else {
                                format!("/{}", href)
                            };
                            format!("{}{}", BASE_URL.trim_end_matches('/'), path)
                        };

                        chapters.push(Chapter {
                            id: 0,
                            manga_source_data_id: 0,
                            chapter_number: chapter_title,
                            url,
                            scraped: false,
                        });
                    }
                }
                if !chapters.is_empty() {
                    break;
                }
            }
        }

        Ok(chapters)
    }
}

/// RizzFables - Browser implementation with Cloudflare bypass
pub mod rizzfables_browser {
    use super::*;
    const BASE_URL: &str = "https://rizzfables.com";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;

        // Try series page first, then manga page
        let url_patterns = vec![
            format!("{}/series", BASE_URL),
            format!("{}/manga/?page=1", BASE_URL),
        ];

        for url in url_patterns {
            log::info!("Fetching from {} with Cloudflare bypass", url);

            if let Ok(html) = browser.navigate_with_cloudflare_bypass(&url) {
                let results = super::wp_manga_browser::parse_manga_page(&html, BASE_URL);
                if !results.is_empty() {
                    return Ok(results);
                }
            }
        }

        Ok(Vec::new())
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;

        log::info!("Fetching chapters from {} with Cloudflare bypass", series_url);
        let html = browser.navigate_with_cloudflare_bypass(series_url)?;

        // Parse chapters from the HTML directly
        let document = Html::parse_document(&html);
        let selectors = [
            "li.wp-manga-chapter a",
            "ul.main.version-chap li a",
            "div.listing-chapters_wrap a",
            "div.eplister a",
            "div.bxcl a",
            "div#chapterlist a",
        ];

        let mut chapters = Vec::new();

        for sel in &selectors {
            if let Ok(selector) = Selector::parse(sel) {
                for a in document.select(&selector) {
                    let chapter_title = a.text().collect::<String>().trim().to_string();
                    if let Some(href) = a.value().attr("href") {
                        let url = if href.starts_with("http") {
                            href.to_string()
                        } else {
                            let path = if href.starts_with('/') {
                                href.to_string()
                            } else {
                                format!("/{}", href)
                            };
                            format!("{}{}", BASE_URL.trim_end_matches('/'), path)
                        };

                        chapters.push(Chapter {
                            id: 0,
                            manga_source_data_id: 0,
                            chapter_number: chapter_title,
                            url,
                            scraped: false,
                        });
                    }
                }
                if !chapters.is_empty() {
                    break;
                }
            }
        }

        Ok(chapters)
    }
}

/// SirenScans - Browser implementation with Cloudflare bypass
pub mod sirenscans_browser {
    use super::*;
    const BASE_URL: &str = "https://sirenscans.com";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::search_manga_with_urls_base(BASE_URL).await
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::get_chapters_base(BASE_URL, series_url).await
    }
}

/// VortexScans - Browser implementation with Cloudflare bypass
pub mod vortexscans_browser {
    use super::*;
    const BASE_URL: &str = "https://vortexscans.org";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::search_manga_with_urls_base(BASE_URL).await
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::get_chapters_base(BASE_URL, series_url).await
    }
}

/// GrimScans - Browser implementation with Cloudflare bypass
pub mod grimscans_browser {
    use super::*;
    const BASE_URL: &str = "https://grimscans.com";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::search_manga_with_urls_base(BASE_URL).await
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::get_chapters_base(BASE_URL, series_url).await
    }
}

/// ThunderScans - Browser implementation
pub mod thunderscans_browser {
    use super::*;
    const BASE_URL: &str = "https://en-thunderscans.com";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::search_manga_with_urls_base(BASE_URL).await
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::get_chapters_base(BASE_URL, series_url).await
    }
}

/// TempleScan - Browser implementation
pub mod templescan_browser {
    use super::*;
    const BASE_URL: &str = "https://templetoons.com";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::search_manga_with_urls_base(BASE_URL).await
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        super::wp_manga_browser::get_chapters_base(BASE_URL, series_url).await
    }
}

/// Kagane - Browser implementation for JS-heavy Next.js site
pub mod kagane_browser {
    use super::*;
    const BASE_URL: &str = "https://kagane.org";

    pub async fn search_manga_with_urls() -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;
        let url = format!("{}/search?sort=avg_views_today%2Cdesc&exg=", BASE_URL);

        log::info!("Fetching from {} using browser for JS rendering", url);
        let html = browser.get_html_wait_for(&url, "a[href*='/series/']", Some(std::time::Duration::from_secs(10)))?;

        let document = Html::parse_document(&html);
        let a_sel = Selector::parse("a").unwrap();
        let mut results = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for a in document.select(&a_sel) {
            if let Some(href) = a.value().attr("href") {
                if href.starts_with("/series/") && seen.insert(href.to_string()) {
                    let title = a.text().collect::<String>().trim().to_string();
                    let series_url = format!("{}{}", BASE_URL, href);

                    // Determine final title, filtering out hash-like slugs
                    let final_title = if title.is_empty() {
                        let slug = href.trim_start_matches("/series/");
                        // Skip hash-like slugs (all uppercase alphanumeric, 20+ chars)
                        if slug.len() > 20 && slug.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
                            continue; // Skip this result entirely
                        }
                        slug.replace(['-', '_'], " ")
                    } else {
                        title
                    };

                    let cover_url = a.select(&Selector::parse("img").unwrap())
                        .next()
                        .and_then(|img| img.value().attr("src").or_else(|| img.value().attr("data-src")))
                        .map(|s| s.to_string());

                    results.push((
                        Manga {
                            id: String::new(),
                            title: final_title,
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

        Ok(results)
    }

    pub async fn get_chapters(series_url: &str) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new().await?;

        log::info!("Fetching chapters from {} using browser", series_url);
        let html = browser.get_html_wait_for(series_url, "a[href*='/reader/']", Some(std::time::Duration::from_secs(10)))?;

        let document = Html::parse_document(&html);
        let a_sel = Selector::parse("a").unwrap();
        let mut chapters = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for a in document.select(&a_sel) {
            if let Some(href) = a.value().attr("href") {
                if (href.contains("/reader/") || href.contains("/chapter/")) && seen.insert(href.to_string()) {
                    let chapter_title = a.text().collect::<String>().trim().to_string();
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("{}{}", BASE_URL, href)
                    };

                    chapters.push(Chapter {
                        id: 0,
                        manga_source_data_id: 0,
                        chapter_number: if chapter_title.is_empty() {
                            href.to_string()
                        } else {
                            chapter_title
                        },
                        url,
                        scraped: false,
                    });
                }
            }
        }

        Ok(chapters)
    }
}
