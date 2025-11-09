use crate::models::{Chapter, Manga};
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;

const BASE_URL: &str = "https://flamecomics.xyz";

#[derive(Deserialize, Debug)]
struct NextData {
    props: NextProps,
}

#[derive(Deserialize, Debug)]
struct NextProps {
    #[serde(rename = "pageProps")]
    page_props: PageProps,
}

#[derive(Deserialize, Debug)]
struct PageProps {
    #[serde(rename = "latestEntries", default)]
    latest_entries: Option<LatestEntries>,

    #[serde(default)]
    series: Option<SeriesData>,

    #[serde(default)]
    chapters: Option<Vec<ChapterData>>,
}

#[derive(Deserialize, Debug)]
struct LatestEntries {
    blocks: Vec<Block>,
}

#[derive(Deserialize, Debug)]
struct Block {
    series: Vec<SeriesData>,
}

#[derive(Deserialize, Debug)]
struct SeriesData {
    series_id: u32,
    title: String,

    #[serde(default, rename = "altTitles")]
    alt_titles: Option<Vec<String>>,

    #[serde(default)]
    description: Option<String>,

    #[serde(default)]
    cover: Option<String>,

    #[serde(default)]
    author: Option<Vec<String>>,

    #[serde(default)]
    artist: Option<Vec<String>>,

    #[serde(default)]
    publisher: Option<Vec<String>>,

    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Deserialize, Debug, Clone)]
struct ChapterData {
    chapter_id: u32,
    series_id: u32,
    chapter: String,

    #[serde(default)]
    title: Option<String>,
}

/// Extract __NEXT_DATA__ JSON from Next.js HTML
fn extract_next_data(html: &str) -> Result<NextData, Box<dyn std::error::Error>> {
    let re = Regex::new(r#"<script id="__NEXT_DATA__" type="application/json">(.+?)</script>"#)?;

    let captures = re
        .captures(html)
        .ok_or("Could not find __NEXT_DATA__ in HTML")?;

    let json_str = captures
        .get(1)
        .ok_or("Could not extract JSON from __NEXT_DATA__")?
        .as_str();

    let data: NextData = serde_json::from_str(json_str)?;
    Ok(data)
}

/// Flame Comics - Free scanlation site (Next.js/React app)
pub async fn search_manga_with_urls(
    client: &Client,
    _title: &str,
) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let url = format!("{}", BASE_URL);
    let html = client.get(&url).send().await?.text().await?;

    let next_data = match extract_next_data(&html) {
        Ok(data) => data,
        Err(e) => {
            log::error!("Failed to extract Next.js data from FlameComics: {}", e);
            return Ok(Vec::new());
        }
    };

    let mut results = Vec::new();

    // Extract series from latestEntries.blocks
    if let Some(latest) = next_data.props.page_props.latest_entries {
        for block in latest.blocks {
            for series in block.series {
                let manga = Manga {
                    id: String::new(),
                    title: series.title.clone(),
                    alt_titles: None,
                    cover_url: series.cover.as_ref().map(|c| {
                        if c.starts_with("http") {
                            c.clone()
                        } else {
                            format!(
                                "https://cdn.flamecomics.xyz/uploads/images/series/{}/{}",
                                series.series_id, c
                            )
                        }
                    }),
                    description: series.description.clone(),
                    tags: series.tags.as_ref().map(|tags| tags.join(", ")),
                    rating: None,
                    monitored: None,
                    check_interval_secs: None,
                    discover_interval_secs: None,
                    last_chapter_check: None,
                    last_discover_check: None,
                };

                let series_url = format!("{}/series/{}", BASE_URL, series.series_id);
                results.push((manga, series_url));

                if results.len() >= 10 {
                    break;
                }
            }

            if results.len() >= 10 {
                break;
            }
        }
    }

    Ok(results)
}

pub async fn get_chapters(
    client: &Client,
    series_url: &str,
) -> Result<Vec<Chapter>, reqwest::Error> {
    let html = client.get(series_url).send().await?.text().await?;

    log::debug!(
        "FlameComics: Fetched {} bytes of HTML from {}",
        html.len(),
        series_url
    );

    let next_data = match extract_next_data(&html) {
        Ok(data) => data,
        Err(e) => {
            log::error!(
                "Failed to extract Next.js data from FlameComics series page: {}",
                e
            );
            log::debug!("HTML preview: {}", &html[..html.len().min(500)]);
            return Ok(Vec::new());
        }
    };

    log::debug!("FlameComics: Successfully extracted Next.js data");

    let mut chapters = Vec::new();

    if let Some(chapter_data) = next_data.props.page_props.chapters {
        log::debug!(
            "FlameComics: Found {} chapters in pageProps",
            chapter_data.len()
        );
        for ch in chapter_data {
            // Build chapter URL
            let chapter_url = format!("{}/series/{}/{}", BASE_URL, ch.series_id, ch.chapter_id);

            // Format chapter number (e.g., "287.00" -> "Chapter 287")
            let chapter_num = ch.chapter.trim_end_matches(".00").trim_end_matches(".0");

            chapters.push(Chapter {
                id: 0,
                manga_source_data_id: 0,
                chapter_number: chapter_num.to_string(),
                url: chapter_url,
                scraped: false,
            });
        }
    }

    // FlameComics returns chapters in reverse order (newest first), reverse it
    chapters.reverse();

    Ok(chapters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_next_data() {
        let html = r#"<html><script id="__NEXT_DATA__" type="application/json">{"props":{"pageProps":{"series":{"series_id":2,"title":"Test"}}}</script></html>"#;
        let data = extract_next_data(html).unwrap();
        assert!(data.props.page_props.series.is_some());
    }
}
