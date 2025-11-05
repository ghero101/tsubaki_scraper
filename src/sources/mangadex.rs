use crate::models::{Chapter, Manga};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

pub const BASE_URL: &str = "https://api.mangadex.org";

#[derive(Deserialize)]
#[allow(dead_code)]
struct MangaList {
    result: String,
    response: String,
    data: Vec<MangaData>,
}

#[derive(Deserialize)]
struct MangaData {
    id: String,
    attributes: MangaAttributes,
    relationships: Vec<Relationship>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Relationship {
    id: String,
    #[serde(rename = "type")]
    rel_type: String,
    attributes: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MangaAttributes {
    title: HashMap<String, String>,
    alt_titles: Vec<HashMap<String, String>>,
    description: HashMap<String, String>,
    is_locked: bool,
    links: HashMap<String, String>,
    original_language: String,
    last_volume: Option<String>,
    last_chapter: Option<String>,
    publication_demographic: Option<String>,
    status: String,
    year: Option<i32>,
    content_rating: String,
    tags: Vec<Tag>,
    state: String,
    chapter_numbers_reset_on_new_volume: bool,
    created_at: String,
    updated_at: String,
    version: i32,
    available_translated_languages: Vec<String>,
    latest_uploaded_chapter: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Tag {
    id: String,
    attributes: TagAttributes,
}

#[derive(Deserialize)]
struct TagAttributes {
    name: HashMap<String, String>,
}

pub async fn search_manga(
    client: &Client,
    title: &str,
    base_url: &str,
) -> Result<Vec<Manga>, Box<dyn std::error::Error>> {
    let url = format!("{}/manga", base_url);
    let response = client
        .get(&url)
        .query(&[("title", title), ("includes[]", "cover_art"), ("limit", "25")])
        .send()
        .await?;
    let text: String = response.text().await?;
    log::info!("MangaDex response: {}", text);
    let response = serde_json::from_str::<MangaList>(&text)?;

    let manga_list = response
        .data
        .into_iter()
        .map(|manga_data| {
            let mut all_titles: Vec<String> = Vec::new();
            for title_text in manga_data.attributes.title.values() {
                if !title_text.is_empty() && !all_titles.contains(title_text) {
                    all_titles.push(title_text.clone());
                }
            }
            for alt_title_map in &manga_data.attributes.alt_titles {
                for title_text in alt_title_map.values() {
                    if !title_text.is_empty() && !all_titles.contains(title_text) {
                        all_titles.push(title_text.clone());
                    }
                }
            }
            let manga_title = manga_data
                .attributes
                .title
                .get("en")
                .cloned()
                .unwrap_or_else(|| {
                    manga_data
                        .attributes
                        .title
                        .values()
                        .next()
                        .cloned()
                        .unwrap_or_default()
                });
            all_titles.retain(|t| t != &manga_title);
            let alt_titles = if all_titles.is_empty() {
                None
            } else {
                Some(all_titles.join(", "))
            };
            let manga_description = manga_data
                .attributes
                .description
                .get("en")
                .cloned()
                .unwrap_or_default();
            let manga_tags = manga_data
                .attributes
                .tags
                .into_iter()
                .filter_map(|tag| tag.attributes.name.get("en").cloned())
                .collect::<Vec<String>>()
                .join(", ");
            let cover_url = manga_data
                .relationships
                .iter()
                .find(|r| r.rel_type == "cover_art")
                .and_then(|cover_rel| {
                    cover_rel
                        .attributes
                        .as_ref()
                        .and_then(|attrs| attrs.get("fileName"))
                        .and_then(|f| f.as_str())
                        .map(|filename| {
                            format!(
                                "https://uploads.mangadex.org/covers/{}/{}",
                                manga_data.id, filename
                            )
                        })
                });

            Manga {
                id: manga_data.id,
                title: manga_title,
                alt_titles,
                cover_url,
                description: Some(manga_description),
                tags: Some(manga_tags),
                rating: Some(manga_data.attributes.content_rating.clone()),
                monitored: None,
                check_interval_secs: None,
                discover_interval_secs: None,
                last_chapter_check: None,
                last_discover_check: None,
            }
        })
        .collect();

    Ok(manga_list)
}

pub async fn search_all_manga(client: &Client, base_url: &str) -> Result<Vec<Manga>, Box<dyn std::error::Error>> {
    let mut out: Vec<Manga> = Vec::new();
    let mut offset = 0u32;
    let limit = 100u32;
    let max_offset = 200u32; // Limit to 200 manga to avoid infinite loops

    loop {
        if offset >= max_offset { break; }

        let url = format!("{}/manga?limit={}&offset={}&includes[]=cover_art", base_url, limit, offset);
        let mut attempt = 0;
        let list = loop {
            attempt += 1;
            let resp = client
                .get(&url)
                .send()
                .await;
            match resp {
                Ok(r) => {
                    let ok = r.error_for_status();
                    match ok {
                        Ok(rr) => {
                            let text = rr.text().await?;
                            match serde_json::from_str::<MangaList>(&text) {
                                Ok(parsed) => {
                                    break parsed.data;
                                }
                                Err(e) => {
                                    log::error!("MangaDex parse error: {}", e);
                                    if attempt >= 3 { break Vec::new(); }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("MangaDex HTTP error: {}", e);
                            if attempt >= 3 { break Vec::new(); }
                        }
                    }
                }
                Err(e) => {
                    log::error!("MangaDex request error: {}", e);
                    if attempt >= 3 { break Vec::new(); }
                }
            }
        };
        if list.is_empty() { break; }
        for md in list { out.push(map_mangadex(md)); }
        offset += limit;
    }
    Ok(out)
}

fn map_mangadex(manga_data: MangaData) -> Manga {
    let mut all_titles: Vec<String> = Vec::new();
    for title_text in manga_data.attributes.title.values() {
        if !title_text.is_empty() && !all_titles.contains(title_text) {
            all_titles.push(title_text.clone());
        }
    }
    for alt_title_map in &manga_data.attributes.alt_titles {
        for title_text in alt_title_map.values() {
            if !title_text.is_empty() && !all_titles.contains(title_text) {
                all_titles.push(title_text.clone());
            }
        }
    }
    let manga_title = manga_data
        .attributes
        .title
        .get("en")
        .cloned()
        .unwrap_or_else(|| {
            manga_data
                .attributes
                .title
                .values()
                .next()
                .cloned()
                .unwrap_or_default()
        });
    all_titles.retain(|t| t != &manga_title);
    let alt_titles = if all_titles.is_empty() { None } else { Some(all_titles.join(", ")) };
    let manga_description = manga_data
        .attributes
        .description
        .get("en")
        .cloned()
        .unwrap_or_default();
    let manga_tags = manga_data
        .attributes
        .tags
        .into_iter()
        .filter_map(|tag| tag.attributes.name.get("en").cloned())
        .collect::<Vec<String>>()
        .join(", ");
    let cover_url = manga_data
        .relationships
        .iter()
        .find(|r| r.rel_type == "cover_art")
        .and_then(|cover_rel| {
            cover_rel
                .attributes
                .as_ref()
                .and_then(|attrs| attrs.get("fileName"))
                .and_then(|f| f.as_str())
                .map(|filename| format!("https://uploads.mangadex.org/covers/{}/{}", manga_data.id, filename))
        });

    Manga {
        id: manga_data.id,
        title: manga_title,
        alt_titles,
        cover_url,
        description: Some(manga_description),
        tags: Some(manga_tags),
        rating: Some(manga_data.attributes.content_rating.clone()),
        monitored: None,
        check_interval_secs: None,
        discover_interval_secs: None,
        last_chapter_check: None,
        last_discover_check: None,
    }
}

pub async fn get_chapters(
    client: &Client,
    manga_id: &str,
) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
    let mut out: Vec<Chapter> = Vec::new();
    let mut offset = 0u32;
    let limit = 100u32;
    loop {
        let url = format!("{}/manga/{}/feed", BASE_URL, manga_id);
        let resp = client
            .get(&url)
            .query(&[("limit", &limit.to_string()), ("offset", &offset.to_string())])
            .send()
            .await?;
        let text = resp.text().await?;
        let data: serde_json::Value = serde_json::from_str(&text)?;
        let arr = data["data"].as_array().cloned().unwrap_or_default();
        if arr.is_empty() { break; }
        for chapter_data in arr {
            out.push(Chapter {
                id: 0,
                manga_source_data_id: 0,
                chapter_number: chapter_data["attributes"]["chapter"].as_str().unwrap_or_default().to_string(),
                url: chapter_data["id"].as_str().unwrap_or_default().to_string(),
                scraped: false,
            });
        }
        offset += limit;
    }
    Ok(out)
}
