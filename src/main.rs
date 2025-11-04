mod config;
mod db;
mod models;
mod scheduler;
mod scraper;
mod sources;
mod crawler;
mod metadata;
use crate::sources::{
    asmotoon, drakecomic, firescans, kagane, kdtnovels, mangadex, reset_scans, rizzcomic, temple_scan, thunderscans,
};
// mod mal;
// mod anilist;

use crate::models::{
    ChapterWithSource, Manga, MangaSourceData, MangaWithSources, PaginatedResponse, PaginationInfo,
    Source, SourceInfo, Stats,
};
use serde::Serialize;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use uuid::Uuid;
use regex::Regex;

fn parse_source(s: &str) -> Option<Source> {
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
        "temple-scan" | "templescan" => Some(Source::TempleScan),
        "thunderscans" | "thunder-scans" => Some(Source::ThunderScans),
        _ => None,
    }
}

fn wp_manga_source_by_name(name: &str) -> Option<(i32, &'static str)> {
    match name {
        "asurascans" => Some((11, "https://asurascans.com")),
        "kenscans" => Some((25, "https://kenscans.com")),
        "sirenscans" | "siren-scans" => Some((43, "https://sirenscans.com")),
        "vortexscans" | "vortex-scans" => Some((56, "https://vortexscans.com")),
        "witchscans" | "witch-scans" => Some((59, "https://witchscans.com")),
        "qiscans" | "qi-scans" => Some((38, "https://qiscans.com")),
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

// Helper function to normalize manga titles for consistent HashMap keys
fn normalize_title(title: &str) -> String {
    title.to_lowercase().replace(" ", "").replace("-", "")
}

// Helper function to merge alt_titles with deduplication
fn merge_alt_titles(existing: &mut Option<String>, new_titles: &str) {
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

use log::{error, info};

#[get("/import")]
async fn import(data: web::Data<AppState>) -> impl Responder {
    info!("Starting import process...");
    let client = &data.client;
    let mut conn = data.db.lock().unwrap();

    let mut manga_map: HashMap<String, Manga> = HashMap::new();
    let mut manga_source_data_map: HashMap<String, Vec<MangaSourceData>> = HashMap::new();

    // Process MangaDex
    info!("Processing MangaDex...");
    let mangadex_manga = match mangadex::search_manga(client, "", mangadex::BASE_URL).await {
        Ok(manga) => manga,
        Err(e) => {
            error!("Failed to fetch manga from MangaDex: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    for manga_item in mangadex_manga {
        let normalized_title = normalize_title(&manga_item.title);
        let current_manga = manga_map
            .entry(normalized_title.clone())
            .or_insert_with(|| {
                let mut new_manga = manga_item.clone();
                new_manga.id = Uuid::new_v4().to_string();
                new_manga
            });

        // Add the source's title to alt_titles if it's different from the main title
        if manga_item.title != current_manga.title {
            merge_alt_titles(&mut current_manga.alt_titles, &manga_item.title);
        }

        // Merge metadata
        if current_manga.cover_url.is_none() && manga_item.cover_url.is_some() {
            current_manga.cover_url = manga_item.cover_url.clone();
        }
        if let Some(alt_titles) = &manga_item.alt_titles {
            merge_alt_titles(&mut current_manga.alt_titles, alt_titles);
        }
        if current_manga.description.is_none() && manga_item.description.is_some() {
            current_manga.description = manga_item.description.clone();
        }
        if let Some(tags) = &manga_item.tags {
            merge_alt_titles(&mut current_manga.tags, tags);
        }

        let manga_source_data = MangaSourceData {
            manga_id: current_manga.id.clone(),
            source_id: Source::MangaDex as i32,
            source_manga_id: manga_item.id.clone(), // MangaDex specific ID
            source_manga_url: format!("https://mangadex.org/title/{}", manga_item.id),
        };
        manga_source_data_map
            .entry(normalized_title.clone())
            .or_default()
            .push(manga_source_data);
    }
    info!("Finished processing MangaDex.");

    // Process Fire Scans
    info!("Processing Fire Scans...");
    let firescans_manga = match firescans::search_manga_with_urls(client, "").await {
        Ok(manga) => manga,
        Err(e) => {
            error!("Failed to fetch manga from Fire Scans: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    for (manga_item, series_url) in firescans_manga {
        let normalized_title = normalize_title(&manga_item.title);
        let current_manga = manga_map
            .entry(normalized_title.clone())
            .or_insert_with(|| {
                let mut new_manga = manga_item.clone();
                new_manga.id = Uuid::new_v4().to_string();
                new_manga
            });

        if manga_item.title != current_manga.title {
            merge_alt_titles(&mut current_manga.alt_titles, &manga_item.title);
        }

        if current_manga.cover_url.is_none() && manga_item.cover_url.is_some() {
            current_manga.cover_url = manga_item.cover_url.clone();
        }
        if let Some(alt_titles) = &manga_item.alt_titles {
            merge_alt_titles(&mut current_manga.alt_titles, alt_titles);
        }

        let manga_source_data = MangaSourceData {
            manga_id: current_manga.id.clone(),
            source_id: Source::FireScans as i32,
            source_manga_id: series_url.clone(),
            source_manga_url: series_url,
        };
        manga_source_data_map
            .entry(normalized_title.clone())
            .or_default()
            .push(manga_source_data);
    }
    info!("Finished processing Fire Scans.");

    // Process Rizz Comic
    info!("Processing Rizz Comic...");
    let rizzcomic_manga = match rizzcomic::search_manga_with_urls(client, "").await {
        Ok(manga) => manga,
        Err(e) => {
            error!("Failed to fetch manga from Rizz Comic: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    for (manga_item, series_url) in rizzcomic_manga {
        let normalized_title = normalize_title(&manga_item.title);
        let current_manga = manga_map
            .entry(normalized_title.clone())
            .or_insert_with(|| {
                let mut new_manga = manga_item.clone();
                new_manga.id = Uuid::new_v4().to_string();
                new_manga
            });

        if manga_item.title != current_manga.title {
            merge_alt_titles(&mut current_manga.alt_titles, &manga_item.title);
        }

        if current_manga.cover_url.is_none() && manga_item.cover_url.is_some() {
            current_manga.cover_url = manga_item.cover_url.clone();
        }
        if let Some(alt_titles) = &manga_item.alt_titles {
            merge_alt_titles(&mut current_manga.alt_titles, alt_titles);
        }

        let manga_source_data = MangaSourceData {
            manga_id: current_manga.id.clone(),
            source_id: Source::RizzComic as i32,
            source_manga_id: series_url.clone(),
            source_manga_url: series_url,
        };
        manga_source_data_map
            .entry(normalized_title.clone())
            .or_default()
            .push(manga_source_data);
    }
    info!("Finished processing Rizz Comic.");

    // Process DrakeComic
    info!("Processing DrakeComic...");
    let drake_manga = match drakecomic::search_manga_with_urls(client, "").await {
        Ok(manga) => manga,
        Err(e) => {
            error!("Failed to fetch manga from DrakeComic: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    for (manga_item, series_url) in drake_manga {
        let normalized_title = normalize_title(&manga_item.title);
        let current_manga = manga_map
            .entry(normalized_title.clone())
            .or_insert_with(|| {
                let mut new_manga = manga_item.clone();
                new_manga.id = Uuid::new_v4().to_string();
                new_manga
            });

        if current_manga.cover_url.is_none() && manga_item.cover_url.is_some() {
            current_manga.cover_url = manga_item.cover_url.clone();
        }

        let manga_source_data = MangaSourceData {
            manga_id: current_manga.id.clone(),
            source_id: Source::DrakeComic as i32,
            source_manga_id: series_url.clone(),
            source_manga_url: series_url,
        };
        manga_source_data_map
            .entry(normalized_title.clone())
            .or_default()
            .push(manga_source_data);
    }
    info!("Finished processing DrakeComic.");

    // Process Asmotoon
    info!("Processing Asmotoon...");
    let asm_manga = match asmotoon::search_manga_with_urls(client, "").await {
        Ok(manga) => manga,
        Err(e) => {
            error!("Failed to fetch manga from Asmotoon: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    for (manga_item, series_url) in asm_manga {
        let normalized_title = normalize_title(&manga_item.title);
        let current_manga = manga_map
            .entry(normalized_title.clone())
            .or_insert_with(|| {
                let mut new_manga = manga_item.clone();
                new_manga.id = Uuid::new_v4().to_string();
                new_manga
            });

        if current_manga.cover_url.is_none() && manga_item.cover_url.is_some() {
            current_manga.cover_url = manga_item.cover_url.clone();
        }

        let manga_source_data = MangaSourceData {
            manga_id: current_manga.id.clone(),
            source_id: Source::Asmotoon as i32,
            source_manga_id: series_url.clone(),
            source_manga_url: series_url,
        };
        manga_source_data_map
            .entry(normalized_title.clone())
            .or_default()
            .push(manga_source_data);
    }
    info!("Finished processing Asmotoon.");

    // Process Reset-Scans
    info!("Processing Reset-Scans...");
    let reset_manga = match reset_scans::search_manga_with_urls(client, "").await {
        Ok(manga) => manga,
        Err(e) => {
            error!("Failed to fetch manga from Reset-Scans: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };
    for (manga_item, series_url) in reset_manga {
        let normalized_title = normalize_title(&manga_item.title);
        let current_manga = manga_map
            .entry(normalized_title.clone())
            .or_insert_with(|| {
                let mut new_manga = manga_item.clone();
                new_manga.id = Uuid::new_v4().to_string();
                new_manga
            });

        if current_manga.cover_url.is_none() && manga_item.cover_url.is_some() {
            current_manga.cover_url = manga_item.cover_url.clone();
        }

        let manga_source_data = MangaSourceData {
            manga_id: current_manga.id.clone(),
            source_id: Source::ResetScans as i32,
            source_manga_id: series_url.clone(),
            source_manga_url: series_url,
        };
        manga_source_data_map
            .entry(normalized_title.clone())
            .or_default()
            .push(manga_source_data);
    }
    info!("Finished processing Reset-Scans.");

    // Process Kagane (aggregator) — best-effort search, may return empty without client-side rendering
    info!("Processing Kagane (aggregator)...");
    let kagane_manga = match kagane::search_manga_with_urls(client, "").await {
        Ok(list) => list,
        Err(e) => {
            error!("Failed to fetch from Kagane: {}", e);
            Vec::new()
        }
    };
    for (manga_item, series_url) in kagane_manga {
        let normalized_title = normalize_title(&manga_item.title);
        let current_manga = manga_map
            .entry(normalized_title.clone())
            .or_insert_with(|| {
                let mut new_manga = manga_item.clone();
                new_manga.id = Uuid::new_v4().to_string();
                new_manga
            });
        if current_manga.cover_url.is_none() && manga_item.cover_url.is_some() {
            current_manga.cover_url = manga_item.cover_url.clone();
        }
        let manga_source_data = MangaSourceData {
            manga_id: current_manga.id.clone(),
            source_id: Source::Kagane as i32,
            source_manga_id: series_url.clone(),
            source_manga_url: series_url,
        };
        manga_source_data_map
            .entry(normalized_title.clone())
            .or_default()
            .push(manga_source_data);
    }
    info!("Finished processing Kagane.");

    // Process KDT Novels (metadata-only)
    info!("Processing KDT Novels...");
    let kdt_manga = match kdtnovels::search_manga_with_urls(client, "").await {
        Ok(manga) => manga,
        Err(e) => {
            error!("Failed to fetch entries from KDT Novels: {}", e);
            Vec::new()
        }
    };
    for (manga_item, series_url) in kdt_manga {
        let normalized_title = normalize_title(&manga_item.title);
        let current_manga = manga_map
            .entry(normalized_title.clone())
            .or_insert_with(|| {
                let mut new_manga = manga_item.clone();
                new_manga.id = Uuid::new_v4().to_string();
                new_manga
            });
        if current_manga.cover_url.is_none() && manga_item.cover_url.is_some() {
            current_manga.cover_url = manga_item.cover_url.clone();
        }
        let manga_source_data = MangaSourceData {
            manga_id: current_manga.id.clone(),
            source_id: Source::KDTNovels as i32,
            source_manga_id: series_url.clone(),
            source_manga_url: series_url,
        };
        manga_source_data_map
            .entry(normalized_title.clone())
            .or_default()
            .push(manga_source_data);
    }
    info!("Finished processing KDT Novels.");

    let tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    // Insert/Update merged manga and their source data and chapters into the database
    info!("Inserting data into the database...");
    info!("Total manga entries to insert: {}", manga_map.len());
    for (normalized_title, manga) in manga_map {
        info!(
            "Inserting manga - ID: {}, Title: {}, Normalized: {}",
            manga.id, manga.title, normalized_title
        );
        if let Err(e) = db::insert_manga(&tx, &manga) {
            error!("Failed to insert manga: {}", e);
            continue;
        }
        info!("Successfully inserted manga with ID: {}", manga.id);

        if let Some(source_data_list) = manga_source_data_map.get(&normalized_title) {
            info!(
                "Found {} source data entries for manga {}",
                source_data_list.len(),
                manga.id
            );
            for manga_source_data in source_data_list {
                info!(
                    "Inserting source data - manga_id: {}, source_id: {}, source_manga_id: {}",
                    manga_source_data.manga_id,
                    manga_source_data.source_id,
                    manga_source_data.source_manga_id
                );
                let manga_source_data_id =
                    match db::insert_manga_source_data(&tx, &manga_source_data) {
                        Ok(id) => id,
                        Err(e) => {
                            error!("Failed to insert manga source data: {}", e);
                            continue;
                        }
                    };

                info!(
                    "Fetching chapters for source_id: {}",
                    manga_source_data.source_id
                );
                let chapters = match manga_source_data.source_id {
                    x if x == Source::MangaDex as i32 => {
                        match mangadex::get_chapters(client, &manga_source_data.source_manga_id)
                            .await
                        {
                            Ok(chapters) => chapters,
                            Err(e) => {
                                error!("Failed to get chapters from MangaDex: {}", e);
                                Vec::new()
                            }
                        }
                    }
                    x if x == Source::FireScans as i32 => {
                        match firescans::get_chapters(client, &manga_source_data.source_manga_url)
                            .await
                        {
                            Ok(chapters) => chapters,
                            Err(e) => {
                                error!("Failed to get chapters from Fire Scans: {}", e);
                                Vec::new()
                            }
                        }
                    }
                    x if x == Source::RizzComic as i32 => {
                        match rizzcomic::get_chapters(client, &manga_source_data.source_manga_url)
                            .await
                        {
                            Ok(chapters) => chapters,
                            Err(e) => {
                                error!("Failed to get chapters from Rizz Comic: {}", e);
                                Vec::new()
                            }
                        }
                    }
                    x if x == Source::DrakeComic as i32 => {
                        match drakecomic::get_chapters(client, &manga_source_data.source_manga_url)
                            .await
                        {
                            Ok(chapters) => chapters,
                            Err(e) => {
                                error!("Failed to get chapters from DrakeComic: {}", e);
                                Vec::new()
                            }
                        }
                    }
                    x if x == Source::Asmotoon as i32 => {
                        match asmotoon::get_chapters(client, &manga_source_data.source_manga_url)
                            .await
                        {
                            Ok(chapters) => chapters,
                            Err(e) => {
                                error!("Failed to get chapters from Asmotoon: {}", e);
                                Vec::new()
                            }
                        }
                    }
                    x if x == Source::ResetScans as i32 => {
                        match reset_scans::get_chapters(client, &manga_source_data.source_manga_url)
                            .await
                        {
                            Ok(chapters) => chapters,
                            Err(e) => {
                                error!("Failed to get chapters from Reset-Scans: {}", e);
                                Vec::new()
                            }
                        }
                    }
                    x if x == Source::KDTNovels as i32 => Vec::new(), // No chapter scraping for novels yet
                    _ => Vec::new(),                                  // Should not happen
                };

                info!("Found {} chapters, inserting into database", chapters.len());
                if let Err(e) = db::insert_chapters(&tx, manga_source_data_id, &chapters) {
                    error!("Failed to insert chapters: {}", e);
                }
            }
        }
    }

    if let Err(e) = tx.commit() {
        error!("Failed to commit transaction: {}", e);
        return HttpResponse::InternalServerError().finish();
    }

    info!("Finished inserting data into the database.");

    info!("Import process finished.");
    HttpResponse::Ok().finish()
}

#[get("/import/source/{source}")]
async fn import_source_endpoint(
    data: web::Data<AppState>,
    source: web::Path<String>,
) -> impl Responder {
    // Try enum sources first, then generic WP-Manga mapping
    let src_opt = parse_source(&source);
    let wp_opt = if src_opt.is_none() { wp_manga_source_by_name(&source.to_lowercase()) } else { None };
    if src_opt.is_none() && wp_opt.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error":"unknown source"}));
    }
    let client = &data.client;
    let mut conn = data.db.lock().unwrap();

    let mut manga_map: HashMap<String, Manga> = HashMap::new();
    let mut manga_source_data_map: HashMap<String, Vec<MangaSourceData>> = HashMap::new();

    let add_entry = |manga_map: &mut HashMap<String, Manga>, msd_map: &mut HashMap<String, Vec<MangaSourceData>>, m: Manga, source_id: i32, series_url_or_id: String| {
        let normalized_title = normalize_title(&m.title);
        let current_manga = manga_map
            .entry(normalized_title.clone())
            .or_insert_with(|| {
                let mut new_manga = m.clone();
                new_manga.id = Uuid::new_v4().to_string();
                new_manga
            });
        if current_manga.cover_url.is_none() && m.cover_url.is_some() {
            current_manga.cover_url = m.cover_url.clone();
        }
        let msd = MangaSourceData {
            manga_id: current_manga.id.clone(),
            source_id,
            source_manga_id: series_url_or_id.clone(),
            source_manga_url: series_url_or_id,
        };
        msd_map.entry(normalized_title).or_default().push(msd);
    };

    if let Some(src) = src_opt {
    match src {
        Source::MangaDex => {
            match mangadex::search_manga(client, "", mangadex::BASE_URL).await {
                Ok(list) => for m in list { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::MangaDex as i32, String::new()); },
                Err(e) => { error!("MangaDex import error: {}", e); return HttpResponse::InternalServerError().finish(); }
            }
        }
        Source::FireScans => {
            match firescans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::FireScans as i32, u); },
                Err(e) => { error!("FireScans import error: {}", e); return HttpResponse::InternalServerError().finish(); }
            }
        }
        Source::RizzComic => {
            match rizzcomic::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::RizzComic as i32, u); },
                Err(e) => { error!("RizzComic import error: {}", e); return HttpResponse::InternalServerError().finish(); }
            }
        }
        Source::DrakeComic => {
            match drakecomic::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::DrakeComic as i32, u); },
                Err(e) => { error!("DrakeComic import error: {}", e); return HttpResponse::InternalServerError().finish(); }
            }
        }
        Source::Asmotoon => {
            match asmotoon::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::Asmotoon as i32, u); },
                Err(e) => { error!("Asmotoon import error: {}", e); return HttpResponse::InternalServerError().finish(); }
            }
        }
        Source::ResetScans => {
            match reset_scans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::ResetScans as i32, u); },
                Err(e) => { error!("ResetScans import error: {}", e); return HttpResponse::InternalServerError().finish(); }
            }
        }
        Source::Kagane => {
            match kagane::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::Kagane as i32, u); },
                Err(e) => { error!("Kagane import error: {}", e); return HttpResponse::InternalServerError().finish(); }
            }
        }
        Source::TempleScan => {
            match temple_scan::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::TempleScan as i32, u); },
                Err(e) => { error!("TempleScan import error: {}", e); return HttpResponse::InternalServerError().finish(); }
            }
        }
        Source::ThunderScans => {
            match thunderscans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::ThunderScans as i32, u); },
                Err(e) => { error!("ThunderScans import error: {}", e); return HttpResponse::InternalServerError().finish(); }
            }
        }
        Source::KDTNovels | Source::MyAnimeList | Source::AniList => {
            return HttpResponse::BadRequest().json(serde_json::json!({"error":"metadata-only source not supported here"}))
        }
    }
} else if let Some((sid, base)) = wp_opt {
        let s = source.to_lowercase();
        let res = match s.as_str() {
            "asurascans" => crate::sources::asurascans::search_manga_with_urls(client, "").await.map(|items| (11, items)),
            "kenscans" => crate::sources::kenscans::search_manga_with_urls(client, "").await.map(|items| (25, items)),
            "sirenscans" | "siren-scans" => crate::sources::sirenscans::search_manga_with_urls(client, "").await.map(|items| (43, items)),
            "vortexscans" | "vortex-scans" => crate::sources::vortexscans::search_manga_with_urls(client, "").await.map(|items| (56, items)),
            "witchscans" | "witch-scans" => crate::sources::witchscans::search_manga_with_urls(client, "").await.map(|items| (59, items)),
            "qiscans" | "qi-scans" => crate::sources::qiscans::search_manga_with_urls(client, "").await.map(|items| (38, items)),
            "madarascans" => crate::sources::madarascans::search_manga_with_urls(client, "").await.map(|items| (30, items)),
            "rizzfables" => crate::sources::rizzfables::search_manga_with_urls(client, "").await.map(|items| (39, items)),
            "rokaricomics" | "rokari-comics" => crate::sources::rokaricomics::search_manga_with_urls(client, "").await.map(|items| (40, items)),
            "stonescape" => crate::sources::stonescape::search_manga_with_urls(client, "").await.map(|items| (45, items)),
            "manhuaus" => crate::sources::manhuaus::search_manga_with_urls(client, "").await.map(|items| (31, items)),
            "grimscans" => crate::sources::grimscans::search_manga_with_urls(client, "").await.map(|items| (19, items)),
            "hivetoons" => crate::sources::hivetoons::search_manga_with_urls(client, "").await.map(|items| (20, items)),
            "nyxscans" => crate::sources::nyxscans::search_manga_with_urls(client, "").await.map(|items| (34, items)),
            _ => crate::sources::wp_manga::search_manga_with_urls_base(client, base).await.map(|items| (sid, items)),
        };
        match res {
            Ok((resolved_sid, items)) => { for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, resolved_sid, u); } },
            Err(e) => { error!("wp import error {}: {}", s, e); return HttpResponse::InternalServerError().finish(); }
        }
    }

    let tx = match conn.transaction() { Ok(t)=>t, Err(e)=>{ error!("tx error: {}", e); return HttpResponse::InternalServerError().finish(); } };

    // Insert
    for (key, m) in manga_map.iter() {
        if let Err(e) = db::insert_manga(&tx, m) { error!("insert manga {}: {}", m.title, e); }
        if let Some(msds) = manga_source_data_map.get(key) {
            for msd in msds {
                let msd_id = match db::insert_manga_source_data(&tx, msd) { Ok(id)=>id, Err(e)=>{ error!("insert msd: {}", e); continue } };
                let chapters: Vec<crate::models::Chapter> = match msd.source_id {
                    x if x == Source::MangaDex as i32 => match mangadex::get_chapters(client, &m.id).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    x if x == Source::FireScans as i32 => match firescans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    x if x == Source::RizzComic as i32 => match rizzcomic::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    x if x == Source::DrakeComic as i32 => match drakecomic::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    x if x == Source::Asmotoon as i32 => match asmotoon::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    x if x == Source::ResetScans as i32 => match reset_scans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    x if x == Source::Kagane as i32 => match kagane::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    x if x == Source::TempleScan as i32 => match temple_scan::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    x if x == Source::ThunderScans as i32 => match thunderscans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    11 => match crate::sources::asurascans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    25 => match crate::sources::kenscans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    43 => match crate::sources::sirenscans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    56 => match crate::sources::vortexscans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    59 => match crate::sources::witchscans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    38 => match crate::sources::qiscans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    30 => match crate::sources::madarascans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    39 => match crate::sources::rizzfables::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    40 => match crate::sources::rokaricomics::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    45 => match crate::sources::stonescape::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    31 => match crate::sources::manhuaus::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    19 => match crate::sources::grimscans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    20 => match crate::sources::hivetoons::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    34 => match crate::sources::nyxscans::get_chapters(client, &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                    _ => Vec::new(),
                };
                let _ = db::insert_chapters(&tx, msd_id, &chapters);
            }
        }
    }
    if let Err(e) = tx.commit() { error!("commit error: {}", e); return HttpResponse::InternalServerError().finish(); }

    HttpResponse::Ok().json(serde_json::json!({"source": source.to_string(), "manga": manga_map.len()}))
}

#[get("/import/source/{source}/manga")]
async fn import_source_manga_only(
    data: web::Data<AppState>,
    source: web::Path<String>,
) -> impl Responder {
    let src_opt = parse_source(&source);
    let wp_opt = if src_opt.is_none() { wp_manga_source_by_name(&source.to_lowercase()) } else { None };
    if src_opt.is_none() && wp_opt.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error":"unknown source"}));
    }
    let client = &data.client;
    let mut conn = data.db.lock().unwrap();

    let mut manga_map: HashMap<String, Manga> = HashMap::new();
    let mut manga_source_data_map: HashMap<String, Vec<MangaSourceData>> = HashMap::new();

    let add_entry = |manga_map: &mut HashMap<String, Manga>, msd_map: &mut HashMap<String, Vec<MangaSourceData>>, m: Manga, source_id: i32, series_url_or_id: String| {
        let normalized_title = normalize_title(&m.title);
        let current_manga = manga_map
            .entry(normalized_title.clone())
            .or_insert_with(|| {
                let mut new_manga = m.clone();
                new_manga.id = Uuid::new_v4().to_string();
                new_manga
            });
        if current_manga.cover_url.is_none() && m.cover_url.is_some() {
            current_manga.cover_url = m.cover_url.clone();
        }
        let msd = MangaSourceData {
            manga_id: current_manga.id.clone(),
            source_id,
            source_manga_id: series_url_or_id.clone(),
            source_manga_url: series_url_or_id,
        };
        msd_map.entry(normalized_title).or_default().push(msd);
    };

    let scrape_err = |name: &str, e: &dyn std::fmt::Display| {
        error!("{} import error: {}", name, e);
        HttpResponse::InternalServerError().finish()
    };

    let res: Result<(), HttpResponse> = if let Some(src) = src_opt { match src {
        Source::MangaDex => match mangadex::search_manga(client, "", mangadex::BASE_URL).await {
            Ok(list) => { for m in list { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::MangaDex as i32, String::new()); } Ok(()) },
            Err(e) => return scrape_err("MangaDex", &e),
        },
        Source::FireScans => match firescans::search_manga_with_urls(client, "").await {
            Ok(items) => { for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::FireScans as i32, u); } Ok(()) },
            Err(e) => return scrape_err("FireScans", &e),
        },
        Source::RizzComic => match rizzcomic::search_manga_with_urls(client, "").await {
            Ok(items) => { for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::RizzComic as i32, u); } Ok(()) },
            Err(e) => return scrape_err("RizzComic", &e),
        },
        Source::DrakeComic => match drakecomic::search_manga_with_urls(client, "").await {
            Ok(items) => { for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::DrakeComic as i32, u); } Ok(()) },
            Err(e) => return scrape_err("DrakeComic", &e),
        },
        Source::Asmotoon => match asmotoon::search_manga_with_urls(client, "").await {
            Ok(items) => { for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::Asmotoon as i32, u); } Ok(()) },
            Err(e) => return scrape_err("Asmotoon", &e),
        },
        Source::ResetScans => match reset_scans::search_manga_with_urls(client, "").await {
            Ok(items) => { for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::ResetScans as i32, u); } Ok(()) },
            Err(e) => return scrape_err("ResetScans", &e),
        },
        Source::Kagane => match kagane::search_manga_with_urls(client, "").await {
            Ok(items) => { for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::Kagane as i32, u); } Ok(()) },
            Err(e) => return scrape_err("Kagane", &e),
        },
        Source::TempleScan => match temple_scan::search_manga_with_urls(client, "").await {
            Ok(items) => { for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::TempleScan as i32, u); } Ok(()) },
            Err(e) => return scrape_err("TempleScan", &e),
        },
        Source::ThunderScans => match thunderscans::search_manga_with_urls(client, "").await {
            Ok(items) => { for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, Source::ThunderScans as i32, u); } Ok(()) },
            Err(e) => return scrape_err("ThunderScans", &e),
        },
        Source::KDTNovels | Source::MyAnimeList | Source::AniList => return HttpResponse::BadRequest().json(serde_json::json!({"error":"metadata-only source not supported here"})),
    }
    }
    else {
        let (sid, base) = wp_opt.unwrap();
        let s = source.to_lowercase();
        let res = match s.as_str() {
            "asurascans" => crate::sources::asurascans::search_manga_with_urls(client, "").await.map(|items| (11, items)),
            "kenscans" => crate::sources::kenscans::search_manga_with_urls(client, "").await.map(|items| (25, items)),
            "sirenscans" | "siren-scans" => crate::sources::sirenscans::search_manga_with_urls(client, "").await.map(|items| (43, items)),
            "vortexscans" | "vortex-scans" => crate::sources::vortexscans::search_manga_with_urls(client, "").await.map(|items| (56, items)),
            "witchscans" | "witch-scans" => crate::sources::witchscans::search_manga_with_urls(client, "").await.map(|items| (59, items)),
            "qiscans" | "qi-scans" => crate::sources::qiscans::search_manga_with_urls(client, "").await.map(|items| (38, items)),
            "madarascans" => crate::sources::madarascans::search_manga_with_urls(client, "").await.map(|items| (30, items)),
            "rizzfables" => crate::sources::rizzfables::search_manga_with_urls(client, "").await.map(|items| (39, items)),
            "rokaricomics" | "rokari-comics" => crate::sources::rokaricomics::search_manga_with_urls(client, "").await.map(|items| (40, items)),
            "stonescape" => crate::sources::stonescape::search_manga_with_urls(client, "").await.map(|items| (45, items)),
            "manhuaus" => crate::sources::manhuaus::search_manga_with_urls(client, "").await.map(|items| (31, items)),
            "grimscans" => crate::sources::grimscans::search_manga_with_urls(client, "").await.map(|items| (19, items)),
            "hivetoons" => crate::sources::hivetoons::search_manga_with_urls(client, "").await.map(|items| (20, items)),
            "nyxscans" => crate::sources::nyxscans::search_manga_with_urls(client, "").await.map(|items| (34, items)),
            _ => crate::sources::wp_manga::search_manga_with_urls_base(client, base).await.map(|items| (sid, items)),
        };
        match res {
            Ok((resolved_sid, items)) => { for (m,u) in items { add_entry(&mut manga_map, &mut manga_source_data_map, m, resolved_sid, u); } Ok(()) },
            Err(e) => return scrape_err(&s, &e),
        }
    };
    if res.is_err() { return res.unwrap_err(); }

    let tx = match conn.transaction() { Ok(t)=>t, Err(e)=>{ error!("tx error: {}", e); return HttpResponse::InternalServerError().finish(); } };
    for (key, m) in manga_map.iter() {
        let _ = db::insert_manga(&tx, m);
        if let Some(msds) = manga_source_data_map.get(key) {
            for msd in msds { let _ = db::insert_manga_source_data(&tx, msd); }
        }
    }
    if let Err(e) = tx.commit() { error!("commit error: {}", e); return HttpResponse::InternalServerError().finish(); }

    HttpResponse::Ok().json(serde_json::json!({"source": source.to_string(), "manga": manga_map.len(), "chapters": 0}))
}
#[get("/manga")]
async fn list_manga(
    data: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let conn = data.db.lock().unwrap();

    // Parse pagination parameters
    let limit = query
        .get("limit")
        .and_then(|l| l.parse::<i32>().ok())
        .unwrap_or(20);
    let offset = query
        .get("offset")
        .and_then(|o| o.parse::<i32>().ok())
        .unwrap_or(0);
    let sort_by = query.get("sort").map(|s| s.as_str()).unwrap_or("title");
    let rating = query.get("rating").map(|r| r.as_str());

    let (manga_list, total) = if let Some(search) = query.get("search") {
        let tags = query.get("tags").map(|t| t.as_str());
        match db::search_manga_paginated(
            &conn,
            search,
            tags,
            rating,
            Some(limit),
            Some(offset),
            sort_by,
        ) {
            Ok(list) => {
                let total = match db::get_manga_count(&conn) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to get manga count: {}", e);
                        return HttpResponse::InternalServerError()
                            .json(serde_json::json!({"error": "Internal server error"}));
                    }
                };
                (list, total)
            }
            Err(e) => {
                error!("Failed to search manga: {}", e);
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Internal server error"}));
            }
        }
    } else {
        match db::get_manga_paginated(&conn, Some(limit), Some(offset), sort_by, rating) {
            Ok(list) => {
                let total = match db::get_manga_count(&conn) {
                    Ok(c) => c,
                    Err(e) => {
                        error!("Failed to get manga count: {}", e);
                        return HttpResponse::InternalServerError()
                            .json(serde_json::json!({"error": "Internal server error"}));
                    }
                };
                (list, total)
            }
            Err(e) => {
                error!("Failed to fetch manga list: {}", e);
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Internal server error"}));
            }
        }
    };

    let response = PaginatedResponse {
        data: manga_list,
        pagination: PaginationInfo {
            total,
            limit,
            offset,
            has_more: offset + limit < total,
        },
    };

    HttpResponse::Ok().json(response)
}

#[get("/manga/{id}")]
async fn get_manga(data: web::Data<AppState>, id: web::Path<String>) -> impl Responder {
    let conn = data.db.lock().unwrap();

    let manga = match db::get_manga_by_id(&conn, &id) {
        Ok(Some(m)) => m,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({"error": "Manga not found"}))
        }
        Err(e) => {
            error!("Database error fetching manga: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Internal server error"}));
        }
    };

    // Get source data
    let source_data_list = match db::get_manga_source_data_by_manga_id(&conn, &id) {
        Ok(list) => list,
        Err(e) => {
            error!("Database error fetching source data: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Internal server error"}));
        }
    };

    // Build sources with names
    let mut sources = Vec::new();
    for source_data in source_data_list {
        let source_name = match db::get_source_name(&conn, source_data.source_id) {
            Ok(name) => name,
            Err(_) => format!("Source {}", source_data.source_id),
        };
        sources.push(SourceInfo {
            source_id: source_data.source_id,
            source_name,
            source_manga_id: source_data.source_manga_id,
            source_manga_url: source_data.source_manga_url,
        });
    }

    let manga_with_sources = MangaWithSources {
        id: manga.id,
        title: manga.title,
        alt_titles: manga.alt_titles,
        cover_url: manga.cover_url,
        description: manga.description,
        tags: manga.tags,
        rating: manga.rating,
        sources,
    };

    HttpResponse::Ok().json(manga_with_sources)
}

#[post("/manga/{id}/monitor")]
async fn monitor_manga(
    data: web::Data<AppState>,
    id: web::Path<String>,
    body: web::Json<crate::models::MonitorRequest>,
) -> impl Responder {
    let conn = data.db.lock().unwrap();
    let _ = db::set_manga_monitoring(
        &conn,
        &id,
        body.monitored,
        body.check_interval_secs,
        body.discover_interval_secs,
    );
    HttpResponse::Ok().finish()
}

#[get("/manga/{id}/chapters")]
async fn get_chapters(data: web::Data<AppState>, id: web::Path<String>) -> impl Responder {
    let conn = data.db.lock().unwrap();

    let manga_source_data_list = match db::get_manga_source_data_by_manga_id(&conn, &id) {
        Ok(list) => list,
        Err(e) => {
            error!("Database error fetching source data: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Internal server error"}));
        }
    };

    if manga_source_data_list.is_empty() {
        return HttpResponse::NotFound()
            .json(serde_json::json!({"error": "Manga not found or has no sources"}));
    }

    let mut all_chapters = Vec::new();
    for manga_source_data in manga_source_data_list {
        let source_name = match db::get_source_name(&conn, manga_source_data.source_id) {
            Ok(name) => name,
            Err(_) => format!("Source {}", manga_source_data.source_id),
        };

        match db::get_chapters_by_manga_source_data_id(
            &conn,
            &manga_source_data.manga_id,
            manga_source_data.source_id,
        ) {
            Ok(chapters) => {
                for chapter in chapters {
                    all_chapters.push(ChapterWithSource {
                        id: chapter.id,
                        chapter_number: chapter.chapter_number,
                        url: chapter.url,
                        scraped: chapter.scraped,
                        source_id: manga_source_data.source_id,
                        source_name: source_name.clone(),
                    });
                }
            }
            Err(e) => {
                error!("Database error fetching chapters: {}", e);
                // Continue to next source instead of failing completely
            }
        }
    }
    HttpResponse::Ok().json(all_chapters)
}

#[get("/download/{manga_id}/{chapter_number}")]
async fn download(
    data: web::Data<AppState>,
    path: web::Path<(String, String)>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let (manga_id, chapter_number) = path.into_inner();
    let conn = data.db.lock().unwrap();
    let stream = query.get("stream").map(|s| s == "true").unwrap_or(false);

    let manga_source_data_list = match db::get_manga_source_data_by_manga_id(&conn, &manga_id) {
        Ok(list) => list,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Database error"}))
        }
    };

    if manga_source_data_list.len() == 1 {
        let source_data = &manga_source_data_list[0];
        let chapters = match db::get_chapters_by_manga_source_data_id(
            &conn,
            &source_data.manga_id,
            source_data.source_id,
        ) {
            Ok(c) => c,
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Database error"}))
            }
        };
        let chapter = find_best_chapter_match(&chapters, &chapter_number);

        if let Some(chapter) = chapter {
            if stream {
                // Stream file directly
                match scraper::download_chapter_to_memory(
                    &data.client,
                    source_data.source_id,
                    &chapter.url,
                )
                .await
                {
                    Ok(bytes) => {
                        let manga = match db::get_manga_by_id(&conn, &manga_id) {
                            Ok(Some(m)) => m,
                            _ => return HttpResponse::InternalServerError().finish(),
                        };
                        let filename = format!("{} - {}.cbz", manga.title, chapter.chapter_number);
                        return HttpResponse::Ok()
                            .content_type("application/x-cbz")
                            .insert_header((
                                "Content-Disposition",
                                format!("attachment; filename=\"{}\"", filename),
                            ))
                            .body(bytes);
                    }
                    Err(e) => {
                        error!("Failed to download chapter: {}", e);
                        return HttpResponse::InternalServerError()
                            .json(serde_json::json!({"error": "Download failed"}));
                    }
                }
            } else {
                // Save to disk
                let manga = match db::get_manga_by_id(&conn, &manga_id) {
                    Ok(Some(m)) => m,
                    _ => return HttpResponse::InternalServerError().finish(),
                };
                if let Some(cu) = &manga.cover_url { let _ = scraper::ensure_cover_downloaded(&data.client, &data.config.download_dir, &manga.title, cu, source_data.source_id).await; }
                let comicinfo = build_comicinfo(&manga.title, &chapter.chapter_number, manga.description.as_deref(), manga.tags.as_deref());
                match scraper::download_chapter(&data.client, source_data.source_id, &chapter.url, &manga.title, &chapter.chapter_number, &data.config.download_dir, comicinfo.as_deref()).await {
                    Ok(file_path) => return HttpResponse::Ok().json(serde_json::json!({"message": "Downloaded successfully", "file": file_path})),
                    Err(e) => {
                        error!("Failed to download chapter: {}", e);
                        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Download failed"}));
                    }
                }
            }
        } else {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"error": "Chapter not found"}));
        }
    } else {
        HttpResponse::Ok().json(manga_source_data_list)
    }
}

fn guess_source_id_from_url(u: &str) -> Option<i32> {
    let lu = u.to_lowercase();
    if lu.contains("mangadex.org") { return Some(Source::MangaDex as i32); }
    if lu.contains("firescans") { return Some(Source::FireScans as i32); }
    if lu.contains("rizzcomic") { return Some(Source::RizzComic as i32); }
    if lu.contains("drakecomic") { return Some(Source::DrakeComic as i32); }
    if lu.contains("asmotoon") { return Some(Source::Asmotoon as i32); }
    if lu.contains("reset-scans") || lu.contains("resetscans") { return Some(Source::ResetScans as i32); }
    if lu.contains("kagane.org") { return Some(Source::Kagane as i32); }
    None
}

#[get("/download/byurl")]
async fn download_by_url(
    data: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let manga_id = match query.get("manga_id") { Some(v)=>v, None=>return HttpResponse::BadRequest().json(serde_json::json!({"error":"manga_id is required"})) };
    let url = match query.get("url") { Some(v)=>v, None=>return HttpResponse::BadRequest().json(serde_json::json!({"error":"url is required"})) };
    let stream = query.get("stream").map(|s| s=="true").unwrap_or(false);
    let req_source_id = query.get("source_id").and_then(|s| s.parse::<i32>().ok()).or_else(|| guess_source_id_from_url(url));

    let conn = data.db.lock().unwrap();
    let source_data_list = match db::get_manga_source_data_by_manga_id(&conn, manga_id) {
        Ok(list) => list,
        Err(_) => return HttpResponse::InternalServerError().json(serde_json::json!({"error":"Database error"})),
    };
    let (chosen_source_id, _has_msd) = if let Some(sid) = req_source_id {
        // Prefer requested/guessed source id, even if no msd exists for this manga
        if source_data_list.iter().any(|s| s.source_id == sid) { (sid, true) } else { (sid, false) }
    } else if source_data_list.len() == 1 { (source_data_list[0].source_id, true) } else { (0, false) };
    if chosen_source_id == 0 {
        return HttpResponse::BadRequest().json(serde_json::json!({"error":"source could not be determined; specify source_id"}));
    }

    if stream {
        match scraper::download_chapter_to_memory(&data.client, chosen_source_id, url).await {
            Ok(bytes) => HttpResponse::Ok()
                .content_type("application/x-cbz")
                .insert_header(("Content-Disposition", format!("attachment; filename=\"{} - byurl.cbz\"", manga_id)))
                .body(bytes),
            Err(e) => {
                error!("Failed to download chapter by url: {}", e);
                HttpResponse::InternalServerError().json(serde_json::json!({"error":"Download failed"}))
            }
        }
    } else {
        let manga = match db::get_manga_by_id(&conn, manga_id) { Ok(Some(m))=>m, _=>return HttpResponse::InternalServerError().finish() };
        if let Some(cu) = &manga.cover_url { let _ = scraper::ensure_cover_downloaded(&data.client, &data.config.download_dir, &manga.title, cu, chosen_source_id).await; }
        let comicinfo = build_comicinfo(&manga.title, "byurl", manga.description.as_deref(), manga.tags.as_deref());
        match scraper::download_chapter(&data.client, chosen_source_id, url, &manga.title, "byurl", &data.config.download_dir, comicinfo.as_deref()).await {
            Ok(file_path) => HttpResponse::Ok().json(serde_json::json!({"message":"Downloaded successfully","file": file_path})),
            Err(e) => { error!("Failed to download chapter by url: {}", e); HttpResponse::InternalServerError().json(serde_json::json!({"error":"Download failed"})) }
        }
    }
}

#[get("/stats")]
async fn get_stats(data: web::Data<AppState>) -> impl Responder {
    let conn = data.db.lock().unwrap();

    let total_manga = match db::get_manga_count(&conn) {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to get manga count: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Internal server error"}));
        }
    };

    let total_chapters = match db::get_chapter_count(&conn) {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to get chapter count: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Internal server error"}));
        }
    };

    let total_sources = match db::get_source_count(&conn) {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to get source count: {}", e);
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Internal server error"}));
        }
    };

    let stats = Stats {
        total_manga,
        total_chapters,
        total_sources,
    };
    let per_source = match db::get_per_source_counts(&conn) {
        Ok(v) => v,
        Err(_) => Vec::new(),
    };

    HttpResponse::Ok().json(serde_json::json!({
        "totals": stats,
        "per_source": per_source
    }))
}

#[get("/download/{manga_id}/{chapter_number}/{source_id}")]
async fn download_from_source(
    data: web::Data<AppState>,
    path: web::Path<(String, String, i32)>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let (manga_id, chapter_number, source_id) = path.into_inner();
    let conn = data.db.lock().unwrap();
    let stream = query.get("stream").map(|s| s == "true").unwrap_or(false);

    let manga_source_data_list = match db::get_manga_source_data_by_manga_id(&conn, &manga_id) {
        Ok(list) => list,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"error": "Database error"}))
        }
    };
    let source_data = manga_source_data_list
        .iter()
        .find(|s| s.source_id == source_id);

    if let Some(source_data) = source_data {
        let chapters = match db::get_chapters_by_manga_source_data_id(
            &conn,
            &source_data.manga_id,
            source_data.source_id,
        ) {
            Ok(c) => c,
            Err(_) => {
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({"error": "Database error"}))
            }
        };
        let chapter = chapters.iter().find(|c| c.chapter_number == chapter_number);

        if let Some(chapter) = chapter {
            if stream {
                // Stream file directly
                match scraper::download_chapter_to_memory(
                    &data.client,
                    source_data.source_id,
                    &chapter.url,
                )
                .await
                {
                    Ok(bytes) => {
                        let manga = match db::get_manga_by_id(&conn, &manga_id) {
                            Ok(Some(m)) => m,
                            _ => return HttpResponse::InternalServerError().finish(),
                        };
                        let filename = format!("{} - {}.cbz", manga.title, chapter.chapter_number);
                        HttpResponse::Ok()
                            .content_type("application/x-cbz")
                            .insert_header((
                                "Content-Disposition",
                                format!("attachment; filename=\"{}\"", filename),
                            ))
                            .body(bytes)
                    }
                    Err(e) => {
                        error!("Failed to download chapter: {}", e);
                        HttpResponse::InternalServerError()
                            .json(serde_json::json!({"error": "Download failed"}))
                    }
                }
            } else {
                // Save to disk
                let manga = match db::get_manga_by_id(&conn, &manga_id) {
                    Ok(Some(m)) => m,
                    _ => return HttpResponse::InternalServerError().finish(),
                };
                if let Some(cu) = &manga.cover_url { let _ = scraper::ensure_cover_downloaded(&data.client, &data.config.download_dir, &manga.title, cu, source_data.source_id).await; }
                let comicinfo = build_comicinfo(&manga.title, &chapter.chapter_number, manga.description.as_deref(), manga.tags.as_deref());
                match scraper::download_chapter(&data.client, source_data.source_id, &chapter.url, &manga.title, &chapter.chapter_number, &data.config.download_dir, comicinfo.as_deref()).await {
                    Ok(file_path) => HttpResponse::Ok().json(serde_json::json!({"message": "Downloaded successfully", "file": file_path})),
                    Err(e) => {
                        error!("Failed to download chapter: {}", e);
                        HttpResponse::InternalServerError().json(serde_json::json!({"error": "Download failed"}))
                    }
                }
            }
        } else {
            HttpResponse::NotFound().json(serde_json::json!({"error": "Chapter not found"}))
        }
    } else {
        HttpResponse::NotFound().json(serde_json::json!({"error": "Source not found"}))
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

fn build_comicinfo(series: &str, number: &str, summary: Option<&str>, tags: Option<&str>) -> Option<String> {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<ComicInfo>\n");
    xml.push_str(&format!("  <Series>{}</Series>\n", xml_escape(series)));
    xml.push_str(&format!("  <Number>{}</Number>\n", xml_escape(number)));
    if let Some(s) = summary { if !s.trim().is_empty() { xml.push_str(&format!("  <Summary>{}</Summary>\n", xml_escape(s))); } }
    if let Some(g) = tags { if !g.trim().is_empty() { xml.push_str(&format!("  <Genre>{}</Genre>\n", xml_escape(g))); } }
    xml.push_str("</ComicInfo>\n");
    Some(xml)
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct MetadataProgress {
    pub in_progress: bool,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub current_phase: Option<String>,
    pub total_pending: Option<i64>,
    pub processed_in_phase: usize,
    pub mangabaka_updated: usize,
    pub mal_updated: usize,
    pub anilist_updated: usize,
    pub merged_updated: usize,
    pub last_heartbeat: Option<i64>,
    pub error: Option<String>,
}

pub struct AppState {
    db: Mutex<Connection>,
    client: Client,
    config: crate::config::Config,
    crawl_progress: Mutex<crawler::CrawlProgress>,
    metadata_progress: Mutex<MetadataProgress>,
    metadata_cancel: Mutex<bool>,
}

fn normalize_chapter_str(s: &str) -> String {
    let lowered = s.to_lowercase();
    let collapsed = lowered.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed.trim().to_string()
}

fn extract_number(s: &str) -> Option<String> {
    static NUM_RE: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r"(?P<num>\d+(?:\.\d+)?)").unwrap()
    });
    NUM_RE.captures(s).and_then(|cap| cap.name("num").map(|m| m.as_str().to_string()))
}

fn find_best_chapter_match<'a>(chapters: &'a [crate::models::Chapter], query: &str) -> Option<&'a crate::models::Chapter> {
    if chapters.is_empty() { return None; }
    let q_norm = normalize_chapter_str(query);
    // 1) exact
    if let Some(ch) = chapters.iter().find(|c| c.chapter_number == query) { return Some(ch); }
    // 2) normalized equality
    if let Some(ch) = chapters.iter().find(|c| normalize_chapter_str(&c.chapter_number) == q_norm) { return Some(ch); }
    // 3) numeric-only match
    if let Some(qnum) = extract_number(&q_norm) {
        if let Some(ch) = chapters.iter().find(|c| extract_number(&c.chapter_number).as_deref() == Some(qnum.as_str())) { return Some(ch); }
        // suffix/prefix contains numerics
        if let Some(ch) = chapters.iter().find(|c| normalize_chapter_str(&c.chapter_number).contains(&qnum)) { return Some(ch); }
    }
    // 4) substring fallback
    chapters.iter().find(|c| normalize_chapter_str(&c.chapter_number).contains(&q_norm))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let conn = db::init_db().unwrap();
    db::create_tables(&conn).unwrap();

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .unwrap();

    let cfg = config::Config::load();

    let data = web::Data::new(AppState {
        db: Mutex::new(conn),
        client,
        config: cfg,
        crawl_progress: Mutex::new(crawler::CrawlProgress::default()),
        metadata_progress: Mutex::new(MetadataProgress::default()),
        metadata_cancel: Mutex::new(false),
    });

    // start background scheduler
    scheduler::spawn(data.clone());

    // Try to bind to an available port starting at 8080
    let mut last_err: Option<std::io::Error> = None;
    for port in 8080..=8090 {
        let data_clone = data.clone();
        let addr = format!("127.0.0.1:{}", port);
        match HttpServer::new(move || {
            App::new()
                .app_data(data_clone.clone())
                .service(import)
            .service(list_manga)
            .service(get_manga)
            .service(get_chapters)
            .service(get_stats)
            .service(download)
            .service(download_from_source)
            .service(monitor_manga)
            .service(import_source_endpoint)
            .service(import_source_manga_only)
            .service(download_by_url)
            .route("/import/source/{source}/quick", web::get().to(|data: web::Data<AppState>, source: web::Path<String>, query: web::Query<std::collections::HashMap<String,String>>| async move {
                use serde_json::json;
                // Quick import for WP-Manga: first page only, limited manga and chapters
                let limit_manga = query.get("limit").and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);
                let limit_ch = query.get("chapters").and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                fn wp_base(name: &str) -> Option<String> {
                    let n = name.to_lowercase();
                    match n.as_str() {
                        "firescans" => Some("https://firescans.xyz".to_string()),
                        "rizzcomic" => Some("https://rizzcomic.com".to_string()),
                        "drakecomic" => Some("https://drakecomic.org".to_string()),
                        "asmotoon" => Some("https://asmotoon.com".to_string()),
                        "reset-scans" | "resetscans" => Some("https://reset-scans.org".to_string()),
                        "temple-scan" | "templescan" => Some("https://templescan.net".to_string()),
                        "thunderscans" | "thunder-scans" => Some("https://thunderscans.com".to_string()),
                        _ => None,
                    }
                }
                let base = wp_base(&source).or_else(|| wp_manga_source_by_name(&source).map(|(_,b)| b.to_string()));
                if base.is_none() { return HttpResponse::BadRequest().json(json!({"error":"unknown or non-wp source"})); }
                let base = base.unwrap();
                let client = &data.client;
                let mut conn = data.db.lock().unwrap();
                let mut manga_added = 0usize;
                let mut ch_added = 0usize;
                let tx = match conn.transaction() { Ok(t)=>t, Err(_)=>{ return HttpResponse::InternalServerError().finish(); } };
                // Choose per-source first-page + chapters strategy
                let sname = source.to_lowercase();
                let items: Vec<(Manga,String)> = match sname.as_str() {
                    "firescans" => match crate::sources::firescans::search_manga_first_page(client).await { Ok(v)=>v, Err(e)=>{ return HttpResponse::InternalServerError().json(json!({"error":format!("fetch failed: {}", e)})); } },
                    "rizzcomic" => match crate::sources::rizzcomic::search_manga_first_page(client).await { Ok(v)=>v, Err(e)=>{ return HttpResponse::InternalServerError().json(json!({"error":format!("fetch failed: {}", e)})); } },
                    _ => match crate::sources::wp_manga::search_manga_first_page(client, &base).await { Ok(v)=>v, Err(e)=>{ return HttpResponse::InternalServerError().json(json!({"error":format!("fetch failed: {}", e)})); } },
                };
                for (m,u) in items.into_iter().take(limit_manga) {
                    let mut mm = m.clone(); mm.id = Uuid::new_v4().to_string();
                    let _ = db::insert_manga(&tx, &mm);
                    let sid = if let Some(s) = parse_source(&source) { s as i32 } else if let Some((id,_)) = wp_manga_source_by_name(&source.to_lowercase()) { id } else { 0 };
                    let msd = MangaSourceData { manga_id: mm.id.clone(), source_id: sid, source_manga_id: u.clone(), source_manga_url: u.clone() };
                    let msd_id = match db::insert_manga_source_data(&tx, &msd) { Ok(id)=>id, Err(_)=>continue };
                    let chs = match sname.as_str() {
                        "firescans" => crate::sources::firescans::get_chapters(client, &u).await.unwrap_or_default(),
                        "rizzcomic" => crate::sources::rizzcomic::get_chapters(client, &u).await.unwrap_or_default(),
                        _ => crate::sources::wp_manga::get_chapters_base(client, &base, &u).await.unwrap_or_default(),
                    };
                    let chs_limited: Vec<_> = chs.into_iter().take(limit_ch).collect();
                    let _ = db::insert_chapters(&tx, msd_id, &chs_limited);
                    manga_added += 1; ch_added += chs_limited.len();
                }
                let _ = tx.commit();
                HttpResponse::Ok().json(json!({"source": source.to_string(), "manga_added": manga_added, "chapters_added": ch_added}))
            }))
.route("/crawl/full", web::get().to(|data: web::Data<AppState>, query: web::Query<std::collections::HashMap<String,String>>| async move {
                fn name_to_id(name: &str) -> Option<i32> {
                    if let Some(s) = parse_source(name) { return Some(s as i32); }
                    if let Some((id,_)) = wp_manga_source_by_name(name) { return Some(id); }
                    name.parse::<i32>().ok()
                }
                let include = query.get("include").map(|s| s.split(',').filter_map(|n| name_to_id(n.trim())).collect::<std::collections::HashSet<i32>>());
                let mut exclude = query.get("exclude").map(|s| s.split(',').filter_map(|n| name_to_id(n.trim())).collect::<std::collections::HashSet<i32>>());
                // Default: exclude Kagane unless explicitly included
                if include.is_none() && exclude.is_none() {
                    let mut ex = std::collections::HashSet::new(); ex.insert(crate::models::Source::Kagane as i32); exclude = Some(ex);
                }
                crawler::spawn_full_crawl_with_filters(data.clone(), include, exclude);
                HttpResponse::Accepted().finish()
            }))
            .route("/crawl/status", web::get().to(|data: web::Data<AppState>| async move {
                let st = crawler::get_progress(data.clone());
                HttpResponse::Ok().json(serde_json::to_value(&st).unwrap())
            }))
            .route("/verify/downloads", web::get().to(|data: web::Data<AppState>| async move {
                use serde_json::json;
                use reqwest::Url;
                let conn = data.db.lock().unwrap();
                let mut stmt = conn.prepare("SELECT id, name FROM sources") .unwrap();
                let rows = stmt.query_map([], |row| { Ok((row.get::<_,i32>(0)?, row.get::<_,String>(1)?)) }).unwrap();
                let mut results = Vec::new();
                for r in rows {
                    if let Ok((sid, sname)) = r {
                        let mut stmt2 = conn.prepare("SELECT m.id, c.chapter_number, c.url, msd.source_manga_url FROM manga m JOIN manga_source_data msd ON msd.manga_id=m.id JOIN chapters c ON c.manga_source_data_id=msd.id WHERE msd.source_id = ?1 LIMIT 1").unwrap();
                        let cand = stmt2.query_map([sid], |row| { Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?, row.get::<_,String>(3)?)) });
                        if let Ok(mut it) = cand {
                            if let Some(Ok((mid, ch, url, base))) = it.next() {
                                let abs = if Url::parse(&url).is_ok() { url } else { Url::parse(&base).ok().and_then(|b| b.join(&url).ok()).map(|u| u.to_string()).unwrap_or(url) };
                                let res = scraper::download_chapter_to_memory(&data.client, sid, &abs).await;
                                match res {
                                    Ok(bytes) => results.push(json!({"source_id":sid,"source":sname,"manga_id":mid,"chapter":ch,"ok":true,"bytes":bytes.len()})),
                                    Err(e) => results.push(json!({"source_id":sid,"source":sname,"manga_id":mid,"chapter":ch,"ok":false,"error":e.to_string()})),
                                }
                            } else {
                                results.push(json!({"source_id":sid,"source":sname,"ok":false,"error":"no chapter found"}));
                            }
                        }
                    }
                }
                HttpResponse::Ok().json(json!({"results": results}))
            }))
.route("/metadata/mangabaka/sync", web::post().to(|data: web::Data<AppState>, query: web::Query<std::collections::HashMap<String,String>>| async move {
                let limit = query.get("limit").and_then(|s| s.parse::<usize>().ok());
                let data_clone = data.clone();
                actix_web::rt::spawn(async move {
                    use chrono::Utc;
                    use tokio::time::{sleep, Duration};
                    { let mut c = data_clone.metadata_cancel.lock().unwrap(); *c = false; }
                    let hb_data = data_clone.clone();
                    actix_web::rt::spawn(async move { loop { { let mut p = hb_data.metadata_progress.lock().unwrap(); if !p.in_progress { break; } p.last_heartbeat = Some(Utc::now().timestamp()); } sleep(Duration::from_secs(2)).await; } });
                    {
                        let mut p = data_clone.metadata_progress.lock().unwrap();
                        *p = MetadataProgress { in_progress: true, started_at: Some(Utc::now().timestamp()), finished_at: None, current_phase: Some("mangabaka_sync".into()), total_pending: None, processed_in_phase: 0, ..Default::default() };
                    }
                    let conn = match rusqlite::Connection::open("manga.db") { Ok(c)=>c, Err(e)=>{ error!("open db: {}", e); return; } };
                    if let Ok(mut s) = conn.prepare("SELECT COUNT(1) FROM manga WHERE mangabaka_id IS NULL OR mangabaka_id = ''") { if let Ok(mut rows)=s.query([]) { if let Some(row_res)=rows.next().transpose(){ if let Ok(r)=row_res { let cnt: i64 = r.get(0).unwrap_or(0); let mut p = data_clone.metadata_progress.lock().unwrap(); p.total_pending = Some(cnt); } } } }
                    let sql = if let Some(l)=limit { format!("SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE mangabaka_id IS NULL OR mangabaka_id = '' LIMIT {}", l) } else { "SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE mangabaka_id IS NULL OR mangabaka_id = ''".to_string() };
                    let mut stmt = match conn.prepare(&sql) { Ok(s)=>s, Err(e)=>{ error!("stmt: {}", e); return; } };
                    let rows_iter = match stmt.query_map([], |row| Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?))) { Ok(r)=>r, Err(e)=>{ error!("query: {}", e); return; } };
                    let rows_vec: Vec<(String,String,String)> = rows_iter.filter_map(|r| r.ok()).collect();
                    drop(stmt);
                    let mut updated = 0usize;
                    for (manga_id,title,alts) in rows_vec {
                        // cancel?
                        { if *data_clone.metadata_cancel.lock().unwrap() { let mut p = data_clone.metadata_progress.lock().unwrap(); p.error=Some("cancelled".into()); p.in_progress=false; return; } }
                        match crate::metadata::mangabaka::resolve_id(&data_clone.client, &title, &alts).await { Ok(Some(pid)) => {
                            let _ = conn.execute("UPDATE manga SET mangabaka_id=?1 WHERE id=?2", rusqlite::params![pid, manga_id]);
                            let _ = conn.execute("INSERT OR REPLACE INTO provider_ids (manga_id, provider, provider_id) VALUES (?1,'mangabaka',?2)", rusqlite::params![manga_id, pid]);
                            updated += 1;
                        }, _=>{} }
                        { let mut p = data_clone.metadata_progress.lock().unwrap(); p.processed_in_phase += 1; }
                    }
                    { let mut p = data_clone.metadata_progress.lock().unwrap(); p.mangabaka_updated += updated; p.current_phase=None; p.finished_at=Some(Utc::now().timestamp()); p.in_progress=false; }
                });
                HttpResponse::Accepted().finish()
            }))
            .route("/metadata/mal/sync", web::post().to(|data: web::Data<AppState>, query: web::Query<std::collections::HashMap<String,String>>| async move {
                let limit = query.get("limit").and_then(|s| s.parse::<usize>().ok());
                let data_clone = data.clone();
                actix_web::rt::spawn(async move {
                    use chrono::Utc;
                    use tokio::time::{sleep, Duration};
                    { let mut c = data_clone.metadata_cancel.lock().unwrap(); *c = false; }
                    let hb_data = data_clone.clone();
                    actix_web::rt::spawn(async move { loop { { let mut p = hb_data.metadata_progress.lock().unwrap(); if !p.in_progress { break; } p.last_heartbeat = Some(Utc::now().timestamp()); } sleep(Duration::from_secs(2)).await; } });
                    {
                        let mut p = data_clone.metadata_progress.lock().unwrap();
                        *p = MetadataProgress { in_progress: true, started_at: Some(Utc::now().timestamp()), finished_at: None, current_phase: Some("mal_sync".into()), total_pending: None, processed_in_phase: 0, ..Default::default() };
                    }
                    // independent connection
                    let conn = match rusqlite::Connection::open("manga.db") { Ok(c)=>c, Err(e)=>{ error!("open db: {}", e); return; } };
                    // set pending count
                    if let Ok(mut s) = conn.prepare("SELECT COUNT(1) FROM manga WHERE mal_id IS NULL OR mal_id = 0") { if let Ok(mut rows)=s.query([]) { if let Some(row_res)=rows.next().transpose(){ if let Ok(r)=row_res { let cnt: i64 = r.get(0).unwrap_or(0); let mut p = data_clone.metadata_progress.lock().unwrap(); p.total_pending = Some(cnt); } } } }
                    // fetch rows
                    let sql = if let Some(l)=limit { format!("SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE mal_id IS NULL OR mal_id = 0 LIMIT {}", l) } else { "SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE mal_id IS NULL OR mal_id = 0".to_string() };
                    let mut stmt = match conn.prepare(&sql) { Ok(s)=>s, Err(e)=>{ error!("stmt: {}", e); return; } };
                    let rows = match stmt.query_map([], |row| Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?))) { Ok(r)=>r, Err(e)=>{ error!("query: {}", e); return; } };
                    let mut updated = 0usize;
                    for row in rows { if let Ok((manga_id,title,alts)) = row {
                        { if *data_clone.metadata_cancel.lock().unwrap() { let mut p = data_clone.metadata_progress.lock().unwrap(); p.error=Some("cancelled".into()); p.in_progress=false; return; } }
                        match crate::metadata::mal::resolve_id(&data_clone.client, &title, &alts).await { Ok(Some(mid)) => {
                            let _ = conn.execute("UPDATE manga SET mal_id=?1 WHERE id=?2", rusqlite::params![mid, manga_id]);
                            let _ = conn.execute("INSERT OR REPLACE INTO provider_ids (manga_id, provider, provider_id) VALUES (?1,'mal',?2)", rusqlite::params![manga_id, mid.to_string()]);
                            updated += 1;
                        }, _=>{} }
                        { let mut p = data_clone.metadata_progress.lock().unwrap(); p.processed_in_phase += 1; }
                    } }
                    { let mut p = data_clone.metadata_progress.lock().unwrap(); p.mal_updated += updated; p.current_phase=None; p.finished_at=Some(Utc::now().timestamp()); p.in_progress=false; }
                });
                HttpResponse::Accepted().finish()
            }))
            .route("/metadata/anilist/sync", web::post().to(|data: web::Data<AppState>, query: web::Query<std::collections::HashMap<String,String>>| async move {
                let limit = query.get("limit").and_then(|s| s.parse::<usize>().ok());
                let data_clone = data.clone();
                actix_web::rt::spawn(async move {
                    use chrono::Utc;
                    {
                        let mut p = data_clone.metadata_progress.lock().unwrap();
                        *p = MetadataProgress { in_progress: true, started_at: Some(Utc::now().timestamp()), finished_at: None, current_phase: Some("anilist_sync".into()), total_pending: None, processed_in_phase:0, ..Default::default() };
                    }
                    let conn = match rusqlite::Connection::open("manga.db") { Ok(c)=>c, Err(e)=>{ error!("open db: {}", e); return; } };
                    if let Ok(mut s) = conn.prepare("SELECT COUNT(1) FROM manga WHERE anilist_id IS NULL OR anilist_id = 0") { if let Ok(mut rows)=s.query([]) { if let Some(row_res)=rows.next().transpose(){ if let Ok(r)=row_res { let cnt: i64 = r.get(0).unwrap_or(0); let mut p = data_clone.metadata_progress.lock().unwrap(); p.total_pending = Some(cnt); } } } }
                    let sql = if let Some(l)=limit { format!("SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE anilist_id IS NULL OR anilist_id = 0 LIMIT {}", l) } else { "SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE anilist_id IS NULL OR anilist_id = 0".to_string() };
                    let mut stmt = match conn.prepare(&sql) { Ok(s)=>s, Err(e)=>{ error!("stmt: {}", e); return; } };
                    let rows = match stmt.query_map([], |row| Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?))) { Ok(r)=>r, Err(e)=>{ error!("query: {}", e); return; } };
                    let mut updated = 0usize;
                    for row in rows { if let Ok((manga_id,title,alts)) = row {
                        { if *data_clone.metadata_cancel.lock().unwrap() { let mut p = data_clone.metadata_progress.lock().unwrap(); p.error=Some("cancelled".into()); p.in_progress=false; return; } }
                        match crate::metadata::anilist::resolve_id(&data_clone.client, &title, &alts).await { Ok(Some(aid)) => {
                            let _ = conn.execute("UPDATE manga SET anilist_id=?1 WHERE id=?2", rusqlite::params![aid, manga_id]);
                            let _ = conn.execute("INSERT OR REPLACE INTO provider_ids (manga_id, provider, provider_id) VALUES (?1,'anilist',?2)", rusqlite::params![manga_id, aid.to_string()]);
                            updated += 1;
                        }, _=>{} }
                        { let mut p = data_clone.metadata_progress.lock().unwrap(); p.processed_in_phase += 1; }
                    } }
                    { let mut p = data_clone.metadata_progress.lock().unwrap(); p.anilist_updated += updated; p.current_phase=None; p.finished_at=Some(Utc::now().timestamp()); p.in_progress=false; }
                });
                HttpResponse::Accepted().finish()
            }))
.route("/metadata/cancel", web::post().to(|data: web::Data<AppState>| async move { { let mut c = data.metadata_cancel.lock().unwrap(); *c = true; } HttpResponse::Accepted().finish() }))
            .route("/metadata/aggregate/sync", web::post().to(|data: web::Data<AppState>, query: web::Query<std::collections::HashMap<String,String>>| async move {
                let data_clone = data.clone();
                actix_web::rt::spawn(async move {
                    use chrono::Utc;
                    use tokio::time::{sleep, Duration};
                    let limit = query.get("limit").and_then(|s| s.parse::<usize>().ok());
                    { let mut c = data_clone.metadata_cancel.lock().unwrap(); *c = false; }
                    // heartbeat task
                    let hb_data = data_clone.clone();
                    actix_web::rt::spawn(async move {
                        loop {
                            {
                                let mut p = hb_data.metadata_progress.lock().unwrap();
                                if !p.in_progress { break; }
                                p.last_heartbeat = Some(Utc::now().timestamp());
                            }
                            sleep(Duration::from_secs(2)).await;
                        }
                    });
// step 1: mangabaka (chunked)
                    {
                        let mut p = data_clone.metadata_progress.lock().unwrap();
                        *p = MetadataProgress { in_progress: true, started_at: Some(Utc::now().timestamp()), finished_at: None, current_phase: Some("mangabaka_sync".into()), total_pending: None, processed_in_phase: 0, ..Default::default() };
                    }
                    // estimate pending
                    let conn = match rusqlite::Connection::open("manga.db") { Ok(c)=>c, Err(e)=>{ error!("open db: {}", e); return; } };
                    if let Ok(mut s) = conn.prepare("SELECT COUNT(1) FROM manga WHERE mangabaka_id IS NULL OR mangabaka_id = ''") { if let Ok(mut rows)=s.query([]) { if let Some(row_res)=rows.next().transpose(){ if let Ok(r)=row_res { let cnt: i64 = r.get(0).unwrap_or(0); let mut p = data_clone.metadata_progress.lock().unwrap(); p.total_pending = Some(cnt); } } } }
                    let sql = if let Some(l)=limit { format!("SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE mangabaka_id IS NULL OR mangabaka_id = '' LIMIT {}", l) } else { "SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE mangabaka_id IS NULL OR mangabaka_id = ''".to_string() };
                    let mut stmt = match conn.prepare(&sql) { Ok(s)=>s, Err(e)=>{ error!("stmt: {}", e); return; } };
                    let rows_iter = match stmt.query_map([], |row| Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?))) { Ok(r)=>r, Err(e)=>{ error!("query: {}", e); return; } };
                    let rows_vec: Vec<(String,String,String)> = rows_iter.filter_map(|r| r.ok()).collect();
                    drop(stmt);
                    let mut updated_baka = 0usize;
                    for (manga_id,title,alts) in rows_vec {
                        { if *data_clone.metadata_cancel.lock().unwrap() { let mut p = data_clone.metadata_progress.lock().unwrap(); p.error=Some("cancelled".into()); p.in_progress=false; return; } }
                        match crate::metadata::mangabaka::resolve_id(&data_clone.client, &title, &alts).await { Ok(Some(pid)) => {
                            let _ = conn.execute("UPDATE manga SET mangabaka_id=?1 WHERE id=?2", rusqlite::params![pid, manga_id]);
                            let _ = conn.execute("INSERT OR REPLACE INTO provider_ids (manga_id, provider, provider_id) VALUES (?1,'mangabaka',?2)", rusqlite::params![manga_id, pid]);
                            updated_baka += 1;
                        }, _=>{} }
                        { let mut p = data_clone.metadata_progress.lock().unwrap(); p.processed_in_phase += 1; }
                    }
                    { let mut p = data_clone.metadata_progress.lock().unwrap(); p.mangabaka_updated += updated_baka; }
                    drop(conn);
                    // step 2: mal
                    let conn = match rusqlite::Connection::open("manga.db") { Ok(c)=>c, Err(e)=>{ error!("open db: {}", e); return; } };
                    { let mut p = data_clone.metadata_progress.lock().unwrap(); p.current_phase = Some("mal_sync".into()); p.total_pending=None; p.processed_in_phase=0; }
                    if let Ok(mut s) = conn.prepare("SELECT COUNT(1) FROM manga WHERE mal_id IS NULL OR mal_id = 0") { if let Ok(mut rows)=s.query([]) { if let Some(row_res)=rows.next().transpose(){ if let Ok(r)=row_res { let cnt: i64 = r.get(0).unwrap_or(0); let mut p = data_clone.metadata_progress.lock().unwrap(); p.total_pending = Some(cnt); } } } }
                    // chunked MAL
                    let sql = if let Some(l)=limit { format!("SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE mal_id IS NULL OR mal_id = 0 LIMIT {}", l) } else { "SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE mal_id IS NULL OR mal_id = 0".to_string() };
                    let mut stmt = match conn.prepare(&sql) { Ok(s)=>s, Err(e)=>{ error!("stmt: {}", e); return; } };
                    let rows_iter = match stmt.query_map([], |row| Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?))) { Ok(r)=>r, Err(e)=>{ error!("query: {}", e); return; } };
                    let rows_vec: Vec<(String,String,String)> = rows_iter.filter_map(|r| r.ok()).collect();
                    drop(stmt);
                    let mut updated_mal = 0usize;
                    for (manga_id,title,alts) in rows_vec {
                        { if *data_clone.metadata_cancel.lock().unwrap() { let mut p = data_clone.metadata_progress.lock().unwrap(); p.error=Some("cancelled".into()); p.in_progress=false; return; } }
                        match crate::metadata::mal::resolve_id(&data_clone.client, &title, &alts).await { Ok(Some(mid)) => {
                            let _ = conn.execute("UPDATE manga SET mal_id=?1 WHERE id=?2", rusqlite::params![mid, manga_id]);
                            let _ = conn.execute("INSERT OR REPLACE INTO provider_ids (manga_id, provider, provider_id) VALUES (?1,'mal',?2)", rusqlite::params![manga_id, mid.to_string()]);
                            updated_mal += 1;
                        }, _=>{} }
                        { let mut p = data_clone.metadata_progress.lock().unwrap(); p.processed_in_phase += 1; }
                    }
                    { let mut p = data_clone.metadata_progress.lock().unwrap(); p.mal_updated += updated_mal; }
                    drop(conn);
                    // step 3: anilist
                    let conn = match rusqlite::Connection::open("manga.db") { Ok(c)=>c, Err(e)=>{ error!("open db: {}", e); return; } };
                    { let mut p = data_clone.metadata_progress.lock().unwrap(); p.current_phase = Some("anilist_sync".into()); p.total_pending=None; p.processed_in_phase=0; }
                    if let Ok(mut s) = conn.prepare("SELECT COUNT(1) FROM manga WHERE anilist_id IS NULL OR anilist_id = 0") { if let Ok(mut rows)=s.query([]) { if let Some(row_res)=rows.next().transpose(){ if let Ok(r)=row_res { let cnt: i64 = r.get(0).unwrap_or(0); let mut p = data_clone.metadata_progress.lock().unwrap(); p.total_pending = Some(cnt); } } } }
                    let sql = if let Some(l)=limit { format!("SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE anilist_id IS NULL OR anilist_id = 0 LIMIT {}", l) } else { "SELECT id, title, COALESCE(alt_titles,'') FROM manga WHERE anilist_id IS NULL OR anilist_id = 0".to_string() };
                    let mut stmt = match conn.prepare(&sql) { Ok(s)=>s, Err(e)=>{ error!("stmt: {}", e); return; } };
                    let rows_iter = match stmt.query_map([], |row| Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?))) { Ok(r)=>r, Err(e)=>{ error!("query: {}", e); return; } };
                    let rows_vec: Vec<(String,String,String)> = rows_iter.filter_map(|r| r.ok()).collect();
                    drop(stmt);
                    let mut updated_ani = 0usize;
                    for (manga_id,title,alts) in rows_vec {
                        { if *data_clone.metadata_cancel.lock().unwrap() { let mut p = data_clone.metadata_progress.lock().unwrap(); p.error=Some("cancelled".into()); p.in_progress=false; return; } }
                        match crate::metadata::anilist::resolve_id(&data_clone.client, &title, &alts).await { Ok(Some(aid)) => {
                            let _ = conn.execute("UPDATE manga SET anilist_id=?1 WHERE id=?2", rusqlite::params![aid, manga_id]);
                            let _ = conn.execute("INSERT OR REPLACE INTO provider_ids (manga_id, provider, provider_id) VALUES (?1,'anilist',?2)", rusqlite::params![manga_id, aid.to_string()]);
                            updated_ani += 1;
                        }, _=>{} }
                        { let mut p = data_clone.metadata_progress.lock().unwrap(); p.processed_in_phase += 1; }
                    }
                    { let mut p = data_clone.metadata_progress.lock().unwrap(); p.anilist_updated += updated_ani; }
                    // step 4: merge (fast)
                    let conn = data_clone.db.lock().unwrap();
                    { let mut p = data_clone.metadata_progress.lock().unwrap(); p.current_phase = Some("merge".into()); p.total_pending=None; }
                    match metadata::aggregate::merge_only(&conn, &data_clone.client).await { Ok(n)=>{ let mut p=data_clone.metadata_progress.lock().unwrap(); p.merged_updated=n; p.current_phase=None; p.finished_at=Some(Utc::now().timestamp()); p.in_progress=false; }, Err(e)=>{ let mut p=data_clone.metadata_progress.lock().unwrap(); p.error=Some(format!("merge: {}", e)); p.in_progress=false; p.finished_at=Some(Utc::now().timestamp()); } }
                });
                HttpResponse::Accepted().finish()
            }))
            .route("/metadata/status", web::get().to(|data: web::Data<AppState>| async move {
                let st = data.metadata_progress.lock().unwrap().clone();
                HttpResponse::Ok().json(serde_json::to_value(&st).unwrap())
            }))
            .route("/import/source/{source}/quick", web::get().to(|data: web::Data<AppState>, source: web::Path<String>, query: web::Query<std::collections::HashMap<String,String>>| async move {
                use serde_json::json;
                // Quick import: WP-Manga first page only; limit manga and chapters per manga
                let limit_manga = query.get("limit").and_then(|s| s.parse::<usize>().ok()).unwrap_or(10);
                let limit_ch = query.get("chapters").and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
                // Map to base
                fn wp_base(name: &str) -> Option<String> {
                    let n = name.to_lowercase();
                    match n.as_str() {
                        "firescans" => Some("https://firescans.xyz".to_string()),
                        "rizzcomic" => Some("https://rizzcomic.com".to_string()),
                        "drakecomic" => Some("https://drakecomic.org".to_string()),
                        "asmotoon" => Some("https://asmotoon.com".to_string()),
                        "reset-scans" | "resetscans" => Some("https://reset-scans.org".to_string()),
                        "temple-scan" | "templescan" => Some("https://templescan.net".to_string()),
                        "thunderscans" => Some("https://thunderscans.com".to_string()),
                        _ => None,
                    }
                }
                let base = wp_base(&source).or_else(|| wp_manga_source_by_name(&source).map(|(_,b)| b.to_string()));
                if base.is_none() { return HttpResponse::BadRequest().json(json!({"error":"unknown or non-wp source"})); }
                let base = base.unwrap();
                let client = &data.client;
                let mut conn = data.db.lock().unwrap();
                let mut manga_added = 0usize;
                let mut ch_added = 0usize;
                let tx = match conn.transaction() { Ok(t)=>t, Err(_)=>{ return HttpResponse::InternalServerError().finish(); } };
                // First-page manga
                let items = match crate::sources::wp_manga::search_manga_first_page(client, &base).await { Ok(v)=>v, Err(e)=>{ return HttpResponse::InternalServerError().json(json!({"error":format!("fetch failed: {}", e)})); } };
                for (m,u) in items.into_iter().take(limit_manga) {
                    let mut mm = m.clone(); mm.id = uuid::Uuid::new_v4().to_string();
                    let _ = db::insert_manga(&tx, &mm);
                    let msd = MangaSourceData { manga_id: mm.id.clone(), source_id: 0, source_manga_id: u.clone(), source_manga_url: u.clone() };
                    // Determine source id
                    let sid = if let Some(s) = parse_source(&source) { s as i32 } else if let Some((id,_)) = wp_manga_source_by_name(&source.to_lowercase()) { id } else { 0 };
                    let msd2 = MangaSourceData { source_id: sid, ..msd };
                    let msd_id = match db::insert_manga_source_data(&tx, &msd2) { Ok(id)=>id, Err(_)=>continue };
                    // Chapters (limit)
                    let chs = match crate::sources::wp_manga::get_chapters_base(client, &base, &u).await { Ok(c)=>c, Err(_)=>Vec::new() };
                    let mut chs_limited = Vec::new();
                    for c in chs.into_iter().take(limit_ch) { chs_limited.push(c); }
                    let _ = db::insert_chapters(&tx, msd_id, &chs_limited);
                    manga_added += 1; ch_added += chs_limited.len();
                }
                let _ = tx.commit();
                HttpResponse::Ok().json(json!({"source": source.to_string(), "manga_added": manga_added, "chapters_added": ch_added}))
            }))
            .route("/verify/source/{source}", web::get().to(|data: web::Data<AppState>, source: web::Path<String>| async move {
                use serde_json::json;
                let conn = data.db.lock().unwrap();
                // Map source to id
                let sid = if let Some(s) = parse_source(&source) { s as i32 } else if let Some((id,_)) = wp_manga_source_by_name(&source.to_lowercase()) { id } else { -1 };
                if sid < 0 { return HttpResponse::BadRequest().json(json!({"error":"unknown source"})); }
                // Counts
                let manga_count: i64 = conn.query_row("SELECT COUNT(DISTINCT m.id) FROM manga m JOIN manga_source_data msd ON msd.manga_id=m.id WHERE msd.source_id = ?1", [sid], |row| row.get(0)).unwrap_or(0);
                let chapter_count: i64 = conn.query_row("SELECT COUNT(1) FROM chapters c JOIN manga_source_data msd ON c.manga_source_data_id=msd.id WHERE msd.source_id = ?1", [sid], |row| row.get(0)).unwrap_or(0);
                let meta_count: i64 = conn.query_row("SELECT COUNT(DISTINCT m.id) FROM manga m JOIN manga_source_data msd ON msd.manga_id=m.id WHERE msd.source_id=?1 AND (COALESCE(m.mangabaka_id,'')<>'' OR COALESCE(m.mal_id,0)>0 OR COALESCE(m.anilist_id,0)>0)", [sid], |row| row.get(0)).unwrap_or(0);
                // One download attempt
let mut stmt = conn.prepare("SELECT m.id, c.chapter_number, c.url, msd.source_manga_url FROM manga m JOIN manga_source_data msd ON msd.manga_id=m.id JOIN chapters c ON c.manga_source_data_id=msd.id WHERE msd.source_id = ?1 LIMIT 1").unwrap();
                let mut rows = stmt.query([sid]).unwrap();
let (download_ok, download_error) = if let Ok(Some(row)) = rows.next() {
                    let mid: String = row.get(0).unwrap();
                    let ch: String = row.get(1).unwrap();
                    let url: String = row.get(2).unwrap();
                    let series_url: String = row.get(3).unwrap_or_default();
                    let full_url = if url.starts_with("http") { url.clone() } else { reqwest::Url::parse(&series_url).and_then(|b| b.join(&url)).map(|u| u.to_string()).unwrap_or(url.clone()) };
                    match scraper::download_chapter_to_memory(&data.client, sid, &full_url).await {
                        Ok(bytes) => (true, Some(json!({"manga_id":mid,"chapter":ch,"bytes":bytes.len()}))),
                        Err(e) => (false, Some(json!({"manga_id":mid,"chapter":ch,"error":e.to_string()}))),
                    }
                } else { (false, None) };
                HttpResponse::Ok().json(json!({
                    "source": source.to_string(),
                    "source_id": sid,
                    "manga_count": manga_count,
                    "chapter_count": chapter_count,
                    "metadata_linked_manga": meta_count,
                    "download": { "ok": download_ok, "detail": download_error }
                }))
            }))
        })
        .bind(&addr)
        {
            Ok(server) => {
                info!("Listening on {}", addr);
                return server.run().await;
            }
            Err(e) => {
                last_err = Some(e);
                continue;
            }
        }
    }
    Err(last_err.unwrap_or_else(|| std::io::Error::new(std::io::ErrorKind::AddrInUse, "No available ports 8080-8090")))
}
