use crate::browser::{BrowserManager, BrowserScraper};
use crate::models::{Chapter, Manga};
use scraper::{Html, Selector};
use std::time::Duration;

const BASE_URL: &str = "https://hivetoons.org";

/// Search for manga using browser automation
/// Empty title returns the series list
pub fn search_manga_with_urls_browser(
    manager: &BrowserManager,
    _title: &str,
) -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
    let tab = manager.new_tab()?;
    let scraper = BrowserScraper::with_timeout(tab, Duration::from_secs(30));

    // Navigate to series list
    let list_url = format!("{}/series", BASE_URL);
    scraper.navigate(&list_url)?;

    // Wait for page to load - try multiple selectors
    let wait_selectors = ["a[href*='/series/']", "div", "main", "body"];
    for selector in &wait_selectors {
        if scraper
            .wait_for_selector_with_timeout(selector, Duration::from_secs(5))
            .is_ok()
        {
            break;
        }
    }

    // Additional wait for JavaScript to render content
    std::thread::sleep(Duration::from_secs(3));

    // Scroll to trigger lazy loading
    let _ = scraper.scroll_to_bottom();
    std::thread::sleep(Duration::from_secs(1));

    // Get the rendered HTML
    let html = scraper.get_html()?;

    // Debug: Print HTML length
    eprintln!("DEBUG: HTML length: {} bytes", html.len());

    // Debug: Save HTML to file for inspection
    if let Err(e) = std::fs::write("hivetoons_browser_debug.html", &html) {
        eprintln!("DEBUG: Failed to save HTML: {}", e);
    } else {
        eprintln!("DEBUG: HTML saved to hivetoons_browser_debug.html");
    }

    let document = Html::parse_document(&html);

    let mut results = Vec::new();

    // Try to find series links
    // The page has links like: <a href="https://hivetoons.org/series/SLUG"><h1>TITLE</h1></a>
    if let Ok(link_selector) = Selector::parse("a[href*='/series/']") {
        if let Ok(h1_selector) = Selector::parse("h1") {
            for link in document.select(&link_selector).take(50) {
                if let Some(href) = link.value().attr("href") {
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else {
                        format!("{}{}", BASE_URL, href)
                    };

                    // Skip if not a valid series URL (just /series without a slug)
                    if url == format!("{}/series", BASE_URL) || !url.contains("/series/") {
                        continue;
                    }

                    // Skip chapter URLs
                    if url.contains("/chapter") {
                        continue;
                    }

                    // Check if this link has an h1 child with the title
                    let mut title = String::new();
                    for h1 in link.select(&h1_selector) {
                        title = h1.text().collect::<String>().trim().to_string();
                        break;
                    }

                    // If no h1, try other attributes or text
                    if title.is_empty() {
                        if let Some(title_attr) = link.value().attr("title") {
                            title = title_attr.to_string();
                        } else if let Some(alt_attr) = link.value().attr("alt") {
                            title = alt_attr.to_string();
                        } else {
                            let text = link.text().collect::<String>().trim().to_string();
                            // Only use if it's not too long (avoid capturing all child text)
                            if !text.is_empty() && text.len() < 100 {
                                title = text;
                            }
                        }
                    }

                    if !title.is_empty() && !results.iter().any(|(_, u)| u == &url) {
                        results.push((
                            Manga {
                                id: String::new(),
                                title,
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
                            url,
                        ));

                        // Stop after finding 10 series
                        if results.len() >= 10 {
                            break;
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}

/// Get chapters for a series using browser automation
/// Note: HiveToons doesn't have a dedicated chapter list page per series.
/// Instead, we need to construct an API endpoint or use a different approach.
/// For now, this returns a placeholder.
pub fn get_chapters_browser(
    _manager: &BrowserManager,
    series_url: &str,
) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
    // Extract series slug from URL
    let series_slug = series_url
        .trim_end_matches('/')
        .split('/')
        .last()
        .unwrap_or("");

    eprintln!("DEBUG: Fetching chapters for series: {}", series_slug);

    // HiveToons likely uses an API endpoint to fetch chapters
    // Try constructing the API URL based on common patterns
    let api_patterns = vec![
        format!("{}/api/series/{}/chapters", BASE_URL, series_slug),
        format!("{}/api/chapters/{}", BASE_URL, series_slug),
        format!("https://api.hivetoons.org/series/{}/chapters", series_slug),
        format!(
            "https://dashboard.hivetoons.org/api/series/{}/chapters",
            series_slug
        ),
    ];

    for api_url in &api_patterns {
        eprintln!("DEBUG: Trying API endpoint: {}", api_url);

        // Try fetching from API endpoint using reqwest
        match reqwest::blocking::get(api_url) {
            Ok(response) if response.status().is_success() => {
                eprintln!("DEBUG: Successfully fetched from {}", api_url);
                if let Ok(text) = response.text() {
                    eprintln!(
                        "DEBUG: API response (first 200 chars): {}",
                        text.chars().take(200).collect::<String>()
                    );

                    // Try to parse as JSON
                    // TODO: Implement JSON parsing for chapters
                    // For now, return empty to indicate we found the endpoint
                }
            }
            Ok(response) => {
                eprintln!(
                    "DEBUG: API endpoint {} returned status: {}",
                    api_url,
                    response.status()
                );
            }
            Err(e) => {
                eprintln!("DEBUG: Failed to fetch {}: {}", api_url, e);
            }
        }
    }

    // Fallback: Return a single placeholder chapter
    Ok(vec![Chapter {
        id: 0,
        manga_source_data_id: 0,
        chapter_number: "Chapter 0".to_string(),
        url: format!("{}/chapter-0", series_url.trim_end_matches('/')),
        scraped: false,
    }])
}

/// Extract chapter number from URL or text
fn extract_chapter_number(url: &str, text: &str) -> String {
    // Try to extract from URL first
    if let Some(chapter_part) = url.split('/').last() {
        if chapter_part.starts_with("chapter-") {
            if let Some(num) = chapter_part.strip_prefix("chapter-") {
                return format!("Chapter {}", num);
            }
        }
    }

    // Try regex on URL
    if let Some(captures) = regex::Regex::new(r"chapter[/-](\d+(?:\.\d+)?)")
        .ok()
        .and_then(|re| re.captures(url))
    {
        if let Some(num) = captures.get(1) {
            return format!("Chapter {}", num.as_str());
        }
    }

    // Try to extract from text
    if let Some(captures) = regex::Regex::new(r"(?i)chapter\s*(\d+(?:\.\d+)?)")
        .ok()
        .and_then(|re| re.captures(text))
    {
        if let Some(num) = captures.get(1) {
            return format!("Chapter {}", num.as_str());
        }
    }

    // Fallback to cleaned text
    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_chapter_number() {
        assert_eq!(
            extract_chapter_number("https://example.com/series/title/chapter-5", ""),
            "Chapter 5"
        );
        assert_eq!(
            extract_chapter_number("https://example.com/chapter/10", ""),
            "Chapter 10"
        );
        assert_eq!(extract_chapter_number("", "Chapter 42"), "Chapter 42");
    }
}
