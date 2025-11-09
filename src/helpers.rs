//! Helper functions for the manga scraper API
//!
//! This module provides utility functions used throughout the application:
//! - Source parsing and identification
//! - Title normalization and matching
//! - XML generation for ComicInfo.xml
//! - Chapter number extraction and comparison
//!
//! # Examples
//!
//! ```
//! use rust_manga_scraper::helpers::{parse_source, normalize_title};
//!
//! // Parse source from string
//! let source = parse_source("mangadex");
//! assert!(source.is_some());
//!
//! // Normalize titles for comparison
//! let normalized = normalize_title("One Piece");
//! assert_eq!(normalized, "onepiece");
//! ```

use crate::models::Source;
use std::collections::HashSet;

/// Parse a source name or ID string into a Source enum
pub fn parse_source(s: &str) -> Option<Source> {
    let k = s.to_lowercase();
    if let Ok(n) = k.parse::<i32>() {
        return match n {
            1 => Some(Source::MangaDex),
            2 => Some(Source::FireScans),
            3 => Some(Source::RizzComic),
            4 => Some(Source::MyAnimeList),
            5 => Some(Source::AniList),
            6 => Some(Source::DrakeComic),
            7 => Some(Source::KDTNovels),
            8 => Some(Source::Asmotoon),
            9 => Some(Source::ResetScans),
            10 => Some(Source::Kagane),
            49 => Some(Source::TempleScan),
            50 => Some(Source::ThunderScans),
            _ => None,
        };
    }
    match k.as_str() {
        "mangadex" => Some(Source::MangaDex),
        "firescans" => Some(Source::FireScans),
        "rizzcomic" => Some(Source::RizzComic),
        "myanimelist" | "mal" => Some(Source::MyAnimeList),
        "anilist" => Some(Source::AniList),
        "drakecomic" => Some(Source::DrakeComic),
        "kdtnovels" | "kdt" => Some(Source::KDTNovels),
        "asmotoon" => Some(Source::Asmotoon),
        "resetscans" | "reset-scans" => Some(Source::ResetScans),
        "kagane" => Some(Source::Kagane),
        "temple-scan" | "templescan" | "templetoons" => Some(Source::TempleScan),
        "thunderscans" | "thunder-scans" => Some(Source::ThunderScans),
        _ => None,
    }
}

/// Get source ID and base URL for WP-Manga based sources by name
pub fn wp_manga_source_by_name(name: &str) -> Option<(i32, &'static str)> {
    match name {
        "asurascans" => Some((11, "https://asurascans.com")),
        "kenscans" => Some((25, "https://kenscans.com")),
        "sirenscans" | "siren-scans" => Some((43, "https://sirenscans.com")),
        "vortexscans" | "vortex-scans" => Some((56, "https://vortexscans.com")),
        "witchscans" | "witch-scans" => Some((59, "https://witchscans.com")),
        "qiscans" | "qi-scans" => Some((38, "https://qiscans.org")),
        "madarascans" => Some((30, "https://madarascans.com")),
        "rizzfables" => Some((39, "https://rizzfables.com")),
        "rokaricomics" | "rokari-comics" => Some((40, "https://rokaricomics.com")),
        "stonescape" => Some((45, "https://stonescape.xyz")),
        "manhuaus" => Some((31, "https://manhuaus.com")),
        "grimscans" => Some((19, "https://grimscans.team")),
        "hivetoons" => Some((20, "https://hivetoons.com")),
        "nyxscans" => Some((34, "https://nyxscans.com")),
        _ => None,
    }
}

/// Normalize manga titles for consistent HashMap keys
pub fn normalize_title(title: &str) -> String {
    title.to_lowercase().replace(" ", "").replace("-", "")
}

/// Merge alt_titles with deduplication
pub fn merge_alt_titles(existing: &mut Option<String>, new_titles: &str) {
    let mut title_set: HashSet<String> = HashSet::new();

    // Add existing titles to set
    if let Some(existing_titles) = existing {
        for title in existing_titles.split(", ") {
            if !title.trim().is_empty() {
                title_set.insert(title.trim().to_string());
            }
        }
    }

    // Add new titles to set
    for title in new_titles.split(", ") {
        if !title.trim().is_empty() {
            title_set.insert(title.trim().to_string());
        }
    }

    // Convert set back to comma-separated string
    let mut titles: Vec<String> = title_set.into_iter().collect();
    titles.sort();
    *existing = if titles.is_empty() {
        None
    } else {
        Some(titles.join(", "))
    };
}

/// Escape XML special characters for ComicInfo.xml
pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Build ComicInfo.xml content
pub fn build_comicinfo(
    series: &str,
    number: &str,
    summary: Option<&str>,
    tags: Option<&str>,
) -> Option<String> {
    let series_esc = xml_escape(series);
    let number_esc = xml_escape(number);
    let mut lines = vec![
        r#"<?xml version="1.0"?>"#.to_string(),
        r#"<ComicInfo xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">"#.to_string(),
        format!(r#"  <Series>{}</Series>"#, series_esc),
        format!(r#"  <Number>{}</Number>"#, number_esc),
    ];
    if let Some(s) = summary {
        lines.push(format!(r#"  <Summary>{}</Summary>"#, xml_escape(s)));
    }
    if let Some(t) = tags {
        lines.push(format!(r#"  <Tags>{}</Tags>"#, xml_escape(t)));
    }
    lines.push(r#"</ComicInfo>"#.to_string());
    Some(lines.join("\n"))
}

/// Normalize chapter string for comparison
pub fn normalize_chapter_str(s: &str) -> String {
    s.to_lowercase()
        .replace(" ", "")
        .replace("-", "")
        .replace("_", "")
        .replace("chapter", "")
        .replace("ch", "")
}

/// Extract number from chapter string
pub fn extract_number(s: &str) -> Option<String> {
    let re = regex::Regex::new(r"(\d+(?:\.\d+)?)").ok()?;
    re.captures(s)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

/// Find best matching chapter from a list based on a query string
pub fn find_best_chapter_match<'a>(
    chapters: &'a [crate::models::Chapter],
    query: &str,
) -> Option<&'a crate::models::Chapter> {
    let q_norm = normalize_chapter_str(query);
    // 1) exact normalized match
    if let Some(ch) = chapters
        .iter()
        .find(|c| normalize_chapter_str(&c.chapter_number) == q_norm)
    {
        return Some(ch);
    }
    // 2) numeric match
    if let Some(q_num) = extract_number(query) {
        if let Some(ch) = chapters
            .iter()
            .find(|c| extract_number(&c.chapter_number).as_ref() == Some(&q_num))
        {
            return Some(ch);
        }
    }
    // 3) contains
    if let Some(ch) = chapters.iter().find(|c| {
        c.chapter_number
            .to_lowercase()
            .contains(&query.to_lowercase())
    }) {
        return Some(ch);
    }
    // 4) substring fallback
    chapters
        .iter()
        .find(|c| normalize_chapter_str(&c.chapter_number).contains(&q_norm))
}

/// Guess source ID from a URL
pub fn guess_source_id_from_url(u: &str) -> Option<i32> {
    let u_lower = u.to_lowercase();
    if u_lower.contains("mangadex.org") {
        Some(Source::MangaDex as i32)
    } else if u_lower.contains("firescans") {
        Some(Source::FireScans as i32)
    } else if u_lower.contains("rizzcomic") {
        Some(Source::RizzComic as i32)
    } else {
        None
    }
}
