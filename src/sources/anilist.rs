#![allow(dead_code)]
use reqwest::Client;
use crate::models::{Manga, Chapter};
use serde_json::{json, Value};

const API_URL: &str = "https://graphql.anilist.co";
const BASE_URL: &str = "https://anilist.co";

/// AniList - Metadata source (does not host chapters)
/// Note: This is a metadata/tracking source, not a reading source
pub async fn search_manga_with_urls(client: &Client, title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    // AniList has a public GraphQL API
    // If no search term, get popular manga instead
    let query = if title.is_empty() {
        json!({
            "query": r#"
                query {
                    Page(perPage: 10) {
                        media(type: MANGA, sort: POPULARITY_DESC) {
                            id
                            title {
                                romaji
                                english
                            }
                            description
                            averageScore
                        }
                    }
                }
            "#
        })
    } else {
        json!({
            "query": r#"
                query ($search: String) {
                    Page(perPage: 10) {
                        media(search: $search, type: MANGA) {
                            id
                            title {
                                romaji
                                english
                            }
                            description
                            averageScore
                        }
                    }
                }
            "#,
            "variables": {
                "search": title
            }
        })
    };

    let response = match client
        .post(API_URL)
        .header("Content-Type", "application/json")
        .json(&query)
        .send()
        .await
    {
        Ok(r) => r.json::<Value>().await?,
        Err(e) => {
            log::warn!("AniList API request failed: {}", e);
            return Ok(Vec::new());
        }
    };

    let mut results = Vec::new();

    if let Some(media_list) = response.pointer("/data/Page/media").and_then(|v| v.as_array()) {
        for media in media_list {
            if let Some(id) = media.get("id").and_then(|v| v.as_i64()) {
                let title_romaji = media.pointer("/title/romaji")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let title_english = media.pointer("/title/english")
                    .and_then(|v| v.as_str());

                let title = if !title_romaji.is_empty() {
                    title_romaji.to_string()
                } else if let Some(eng) = title_english {
                    eng.to_string()
                } else {
                    continue;
                };

                let url = format!("{}/manga/{}", BASE_URL, id);
                let description = media.get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let rating = media.get("averageScore")
                    .and_then(|v| v.as_i64())
                    .map(|s| format!("{}/100", s));

                results.push((Manga {
                    id: String::new(),
                    title,
                    alt_titles: title_english.map(|s| s.to_string()),
                    cover_url: None,
                    description,
                    tags: Some("AniList".to_string()),
                    rating,
                    monitored: None,
                    check_interval_secs: None,
                    discover_interval_secs: None,
                    last_chapter_check: None,
                    last_discover_check: None,
                }, url));
            }
        }
    }

    Ok(results)
}

pub async fn get_chapters(_client: &Client, _series_url: &str) -> Result<Vec<Chapter>, reqwest::Error> {
    // AniList is a metadata source, it doesn't host chapters
    Ok(Vec::new())
}
