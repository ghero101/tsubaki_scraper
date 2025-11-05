use crate::models::Source;
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::fs::File;
use std::io::{copy, Cursor};
use std::path::PathBuf;
use zip::write::{FileOptions, ZipWriter};
use regex::Regex;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtHomeServer {
    pub base_url: String,
    pub chapter: AtHomeChapter,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtHomeChapter {
    pub hash: String,
    pub data: Vec<String>,
    #[allow(dead_code)]
    pub data_saver: Vec<String>,
}

fn sanitize_filename(s: &str) -> String {
    s.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
}

fn source_name_from_id(source_id: i32) -> &'static str {
    match source_id {
        x if x == Source::MangaDex as i32 => "MangaDex",
        x if x == Source::FireScans as i32 => "FireScans",
        x if x == Source::RizzComic as i32 => "RizzComic",
        x if x == Source::DrakeComic as i32 => "DrakeComic",
        x if x == Source::Asmotoon as i32 => "Asmotoon",
        x if x == Source::ResetScans as i32 => "ResetScans",
        x if x == Source::Kagane as i32 => "Kagane",
        x if x == Source::KDTNovels as i32 => "KDTNovels",
        _ => "Unknown",
    }
}

fn format_chapter_label(chapter_number: &str, chapter_url: &str) -> String {
    // Try to construct "Vol.X Ch.Y - Title" if possible
    let mut vol: Option<String> = None;
    let mut ch: Option<String> = None;
    let mut title: Option<String> = None;

    let num_re = Regex::new(r"(?i)(?:ch(?:apter)?\s*)?(\d+(?:\.\d+)?)").unwrap();
    let vol_re = Regex::new(r"(?i)(?:vol(?:ume)?\s*)?(\d+)").unwrap();

    // From chapter_number text
    if let Some(cap) = num_re.captures(chapter_number) { ch = cap.get(1).map(|m| m.as_str().to_string()); }
    if let Some(cap) = vol_re.captures(chapter_number) { vol = cap.get(1).map(|m| m.as_str().to_string()); }
    if chapter_number.contains('-') {
        let parts: Vec<&str> = chapter_number.splitn(2, '-').collect();
        if parts.len() == 2 { let t = parts[1].trim(); if !t.is_empty() { title = Some(t.to_string()); } }
    }

    // From URL slug
    let lower_url = chapter_url.to_lowercase();
    if ch.is_none() {
        if let Some(cap) = Regex::new(r"chapter[-/](\d+(?:\.\d+)?)").unwrap().captures(&lower_url) {
            ch = cap.get(1).map(|m| m.as_str().to_string());
        }
    }
    if vol.is_none() {
        if let Some(cap) = Regex::new(r"vol(?:ume)?[-/](\d+)").unwrap().captures(&lower_url) {
            vol = cap.get(1).map(|m| m.as_str().to_string());
        }
    }

    // Build label
    let mut parts: Vec<String> = Vec::new();
    if let Some(v) = vol { parts.push(format!("Vol.{}", v)); }
    if let Some(cn) = ch { parts.push(format!("Ch.{}", cn)); }
    let base = if parts.is_empty() { chapter_number.to_string() } else { parts.join(" ") };
    if let Some(t) = title { format!("{} - {}", base, t) } else { base }
}

pub async fn download_chapter(
    client: &Client,
    source_id: i32,
    chapter_url: &str,
    manga_title: &str,
    chapter_number: &str,
    base_dir: &str,
    comicinfo_xml: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    // Root download directory
    std::fs::create_dir_all(base_dir)?;

    // Per-manga folder with subfolders for covers and artwork
    let sanitized_title = sanitize_filename(manga_title);
    let manga_dir: PathBuf = [base_dir, &sanitized_title].iter().collect();
    std::fs::create_dir_all(&manga_dir)?;
    std::fs::create_dir_all(manga_dir.join("covers"))?;
    std::fs::create_dir_all(manga_dir.join("artwork"))?;

    // Build filename: <sourceId>-<SourceName>-<MangaTitle> - <ChapterLabel>.cbz
    let label = format_chapter_label(chapter_number, chapter_url);
    let sanitized_chapter = sanitize_filename(&label);
    let source_name = sanitize_filename(source_name_from_id(source_id));
    let file_name = format!(
        "{}-{}-{} - {}.cbz",
        source_id, source_name, sanitized_title, sanitized_chapter
    );

    let file_path = manga_dir.join(&file_name);
    let tmp_path = file_path.with_extension("cbz.tmp");
    let file = File::create(&tmp_path)?;
    let mut zip = ZipWriter::new(file);
    if let Some(xml) = comicinfo_xml {
        use std::io::Write;
        zip.start_file("ComicInfo.xml", FileOptions::default())?;
        zip.write_all(xml.as_bytes())?;
    }

    match source_id {
        x if x == Source::MangaDex as i32 => {
            // Get the at-home server URL
            let url = format!("https://api.mangadex.org/at-home/server/{}", chapter_url);
            let response = client
                .get(&url)
                .header("User-Agent", "rust_manga_scraper/0.1.0")
                .send()
                .await?
                .json::<AtHomeServer>()
                .await?;

            // Download each page and add it to the zip file
            for (i, page) in response.chapter.data.iter().enumerate() {
                let page_url = format!(
                    "{}/data/{}/{}",
                    response.base_url, response.chapter.hash, page
                );
                let response = client
                    .get(&page_url)
                    .header("Referer", "https://mangadex.org/")
                    .header("User-Agent", "rust_manga_scraper/0.1.0")
                    .send()
                    .await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        x if x == Source::FireScans as i32
            || x == Source::RizzComic as i32
            || x == Source::DrakeComic as i32
            || x == Source::Asmotoon as i32
            || x == Source::ResetScans as i32
            || x == Source::TempleScan as i32
            || x == Source::ThunderScans as i32 =>
        {
            let response = client
                .get(chapter_url)
                .header("User-Agent", "rust_manga_scraper/0.1.0")
                .send()
                .await?
                .text()
                .await?;
            let document = Html::parse_document(&response);
            let selector = Selector::parse("div.reading-content img").unwrap();
            let mut image_list: Vec<String> = Vec::new();

            for element in document.select(&selector) {
                if let Some(image_url) = element
                    .value()
                    .attr("src")
                    .or_else(|| element.value().attr("data-src"))
                {
                    image_list.push(image_url.to_string());
                }
            }
            if image_list.is_empty() {
                // Regex fallback: scrape direct image URLs from HTML
                let re = Regex::new(r#"https?://[^"'\s>]+\.(?:jpg|jpeg|png)"#).unwrap();
                let mut seen = std::collections::HashSet::new();
                for cap in re.captures_iter(&response) {
                    let url = cap.get(0).unwrap().as_str().to_string();
                    if seen.insert(url.clone()) { image_list.push(url); }
                }
            }

            // Resolve relative image URLs and set a reasonable Referer per origin
            let origin = Url::parse(chapter_url)
                .ok()
                .and_then(|u| Some(format!("{}://{}", u.scheme(), u.host_str()?)))
                .unwrap_or_default();
            for (i, src) in image_list.iter().enumerate() {
                let full_url = if let Ok(base) = Url::parse(chapter_url) {
                    base.join(src).map(|u| u.to_string()).unwrap_or_else(|_| src.clone())
                } else {
                    src.clone()
                };
                let response = client
                    .get(&full_url)
                    .header("Referer", if !origin.is_empty() { origin.as_str() } else { chapter_url })
                    .header("User-Agent", "rust_manga_scraper/0.1.0")
                    .send()
                    .await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        x if x == Source::Kagane as i32 => {
            // Kagane is a Next.js app; attempt to extract all readable images from the reader area.
            let response = client
                .get(chapter_url)
                .header("User-Agent", "rust_manga_scraper/0.1.0")
                .send()
                .await?
                .text()
                .await?;
            let document = Html::parse_document(&response);
            let selectors = vec![
                Selector::parse("div[data-reader] img").unwrap(),
                Selector::parse("div.reading-content img").unwrap(),
                Selector::parse("main img").unwrap(),
            ];
            let mut image_list: Vec<String> = Vec::new();
            'outer: for selector in selectors {
                for element in document.select(&selector) {
                    if let Some(image_url) = element
                        .value()
                        .attr("src")
                        .or_else(|| element.value().attr("data-src"))
                    {
                        // Filter likely page images
                        if image_url.contains("http")
                            && (image_url.ends_with(".jpg")
                                || image_url.ends_with(".jpeg")
                                || image_url.ends_with(".png"))
                        {
                            image_list.push(image_url.to_string());
                        }
                    }
                }
                if !image_list.is_empty() {
                    break 'outer;
                }
            }
            for (i, image_url) in image_list.iter().enumerate() {
                let response = client
                    .get(image_url)
                    .header("Referer", "https://mangadex.org/")
                    .header("User-Agent", "rust_manga_scraper/0.1.0")
                    .send()
                    .await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        _ => {
            // Generic HTML reader fallback
            let response = client.get(chapter_url).send().await?.text().await?;
            let document = Html::parse_document(&response);
            let selector = Selector::parse("div.reading-content img").unwrap();
            let mut image_list: Vec<String> = Vec::new();
            for element in document.select(&selector) {
                if let Some(image_url) = element
                    .value()
                    .attr("src")
                    .or_else(|| element.value().attr("data-src"))
                {
                    image_list.push(image_url.to_string());
                }
            }
            if image_list.is_empty() {
                let re = Regex::new(r#"https?://[^"'\s>]+\.(?:jpg|jpeg|png)"#).unwrap();
                let mut seen = std::collections::HashSet::new();
                for cap in re.captures_iter(&response) {
                    let url = cap.get(0).unwrap().as_str().to_string();
                    if seen.insert(url.clone()) { image_list.push(url); }
                }
            }
            for (i, image_url) in image_list.iter().enumerate() {
                let response = client.get(image_url).send().await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
    }

    let res = zip.finish();
    if let Err(e) = res {
        // Cleanup partial file on error
        let _ = std::fs::remove_file(&tmp_path);
        return Err(Box::new(e));
    }
    // Move temp file to final destination
    std::fs::rename(&tmp_path, &file_path)?;

    Ok(file_path.to_string_lossy().to_string())
}

pub async fn ensure_cover_downloaded(
    client: &Client,
    base_dir: &str,
    manga_title: &str,
    cover_url: &str,
    source_id: i32,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let sanitized_title = sanitize_filename(manga_title);
    let manga_dir: PathBuf = [base_dir, &sanitized_title].iter().collect();
    let covers_dir = manga_dir.join("covers");
    std::fs::create_dir_all(&covers_dir)?;
    // Determine ext
    let ext = cover_url
        .rsplit('.').next().unwrap_or("jpg")
        .split('?').next().unwrap_or("jpg");
    let fname = format!("{}-{}-cover.{}", source_id, sanitize_filename(source_name_from_id(source_id)), ext);
    let path = covers_dir.join(&fname);
    if path.exists() { return Ok(Some(path.to_string_lossy().to_string())); }
    let resp = client.get(cover_url).header("User-Agent", "rust_manga_scraper/0.1.0").send().await?;
    if !resp.status().is_success() { return Ok(None); }
    let bytes = resp.bytes().await?;
    std::fs::write(&path, &bytes)?;
    Ok(Some(path.to_string_lossy().to_string()))
}

pub async fn download_chapter_to_memory(
    client: &Client,
    source_id: i32,
    chapter_url: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use std::io::Cursor;

    let buffer = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buffer);

    match source_id {
        x if x == Source::MangaDex as i32 => {
            let url = format!("https://api.mangadex.org/at-home/server/{}", chapter_url);
            let response = client
                .get(&url)
                .header("User-Agent", "rust_manga_scraper/0.1.0")
                .send()
                .await?
                .json::<AtHomeServer>()
                .await?;

            for (i, page) in response.chapter.data.iter().enumerate() {
                let page_url = format!(
                    "{}/data/{}/{}",
                    response.base_url, response.chapter.hash, page
                );
                let response = client
                    .get(&page_url)
                    .header("Referer", "https://mangadex.org/")
                    .header("User-Agent", "rust_manga_scraper/0.1.0")
                    .send()
                    .await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        x if x == Source::FireScans as i32 || x == Source::RizzComic as i32 => {
            let response = client
                .get(chapter_url)
                .header("User-Agent", "rust_manga_scraper/0.1.0")
                .send()
                .await?
                .text()
                .await?;
            let document = Html::parse_document(&response);
            let selector = Selector::parse("div.reading-content > div.page-break > img").unwrap();
            let mut image_list = Vec::new();

            for element in document.select(&selector) {
                let image_url = element.value().attr("src").unwrap().to_string();
                image_list.push(image_url);
            }

            // Resolve relative image URLs and set a reasonable Referer per origin
            let origin = reqwest::Url::parse(chapter_url)
                .ok()
                .and_then(|u| Some(format!("{}://{}", u.scheme(), u.host_str()?)))
                .unwrap_or_default();
            for (i, src) in image_list.iter().enumerate() {
                let full_url = if let Ok(base) = reqwest::Url::parse(chapter_url) {
                    base.join(src).map(|u| u.to_string()).unwrap_or_else(|_| src.clone())
                } else { src.clone() };
                let response = client
                    .get(&full_url)
                    .header("Referer", if !origin.is_empty() { origin.as_str() } else { chapter_url })
                    .header("User-Agent", "rust_manga_scraper/0.1.0")
                    .send()
                    .await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        x if x == Source::Kagane as i32 => {
            let response = client.get(chapter_url).send().await?.text().await?;
            let document = Html::parse_document(&response);
            let selectors = vec![
                Selector::parse("div[data-reader] img").unwrap(),
                Selector::parse("div.reading-content img").unwrap(),
                Selector::parse("main img").unwrap(),
            ];
            let mut image_list: Vec<String> = Vec::new();
            'outer: for selector in selectors {
                for element in document.select(&selector) {
                    if let Some(image_url) = element
                        .value()
                        .attr("src")
                        .or_else(|| element.value().attr("data-src"))
                    {
                        if image_url.contains("http")
                            && (image_url.ends_with(".jpg")
                                || image_url.ends_with(".jpeg")
                                || image_url.ends_with(".png"))
                        {
                            image_list.push(image_url.to_string());
                        }
                    }
                }
                if !image_list.is_empty() {
                    break 'outer;
                }
            }
            for (i, image_url) in image_list.iter().enumerate() {
                let response = client.get(image_url).send().await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
        _ => {
            // Generic HTML reader fallback to memory
            let response = client.get(chapter_url).send().await?.text().await?;
            let document = Html::parse_document(&response);
            let selector = Selector::parse("div.reading-content img").unwrap();
            let mut image_list = Vec::new();
            for element in document.select(&selector) {
                if let Some(image_url) = element
                    .value()
                    .attr("src")
                    .or_else(|| element.value().attr("data-src"))
                {
                    image_list.push(image_url.to_string());
                }
            }
            let base = reqwest::Url::parse(chapter_url).ok();
            for (i, image_url) in image_list.iter().enumerate() {
                let full_url = if let Some(ref b) = base { b.join(image_url).map(|u| u.to_string()).unwrap_or(image_url.clone()) } else { image_url.clone() };
                let response = client.get(&full_url).send().await?;
                let mut cursor = Cursor::new(response.bytes().await?);
                zip.start_file(format!("page_{}.jpg", i + 1), FileOptions::default())?;
                copy(&mut cursor, &mut zip)?;
            }
        }
    }

    let cursor = zip.finish()?;

    Ok(cursor.into_inner())
}
