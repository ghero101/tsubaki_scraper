use actix_web::web;
use log::{info, error};
use crate::{db, models::{Manga, MangaSourceData, Source, Chapter}};
use std::collections::{HashMap, HashSet};
use tokio::time::{sleep, Duration};
use serde::Serialize;
use chrono::Utc;

fn normalize_title(title: &str) -> String {
    title.to_lowercase().replace(" ", "").replace("-", "")
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct SourceProgress {
    pub name: String,
    pub fetched_manga: usize,
    pub inserted_msd: usize,
}

#[derive(Debug, Default, Serialize, Clone)]
pub struct CrawlProgress {
    pub in_progress: bool,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub current_source: Option<String>,
    pub sources: Vec<SourceProgress>,
    pub error: Option<String>,
}

pub fn spawn_full_crawl(data: web::Data<crate::AppState>) {
    spawn_full_crawl_with_filters(data, None, None);
}

pub fn spawn_full_crawl_with_filters(
    data: web::Data<crate::AppState>,
    include: Option<HashSet<i32>>,
    exclude: Option<HashSet<i32>>,
) {
    let data_clone = data.clone();
    actix_web::rt::spawn(async move {
        info!("Full crawl started...");
        {
            let mut p = data_clone.crawl_progress.lock().unwrap();
            *p = CrawlProgress { in_progress: true, started_at: Some(Utc::now().timestamp()), finished_at: None, current_source: None, sources: Vec::new(), error: None };
        }
        let client = &data_clone.client;
        let mut conn = match data_clone.db.lock() { Ok(c) => c, Err(_) => { error!("DB lock failed"); return; } };
        let tx = match conn.transaction() { Ok(t) => t, Err(e) => { error!("tx error: {}", e); return; } };

        let mut manga_map: HashMap<String, Manga> = HashMap::new();
        let mut msd_map: HashMap<String, Vec<MangaSourceData>> = HashMap::new();

        // Helper to decide if a source id should be crawled
        let allowed = |sid: i32| -> bool {
            if let Some(ref incl) = include { if !incl.contains(&sid) { return false; } }
            if let Some(ref excl) = exclude { if excl.contains(&sid) { return false; } }
            true
        };

        // MangaDex
        if allowed(Source::MangaDex as i32) {
            {
                let mut p = data_clone.crawl_progress.lock().unwrap();
                p.current_source = Some("MangaDex".to_string());
            }
            match crate::sources::mangadex::search_all_manga(client, crate::sources::mangadex::BASE_URL).await {
                Ok(list) => {
                    let mut _fetched = 0usize;
                    for m in list {
                        _fetched += 1;
                        let key = normalize_title(&m.title);
                        let entry = manga_map.entry(key.clone()).or_insert_with(|| Manga { id: uuid::Uuid::new_v4().to_string(), ..m.clone() });
                        if entry.cover_url.is_none() && m.cover_url.is_some() { entry.cover_url = m.cover_url.clone(); }
                        let msd = MangaSourceData { manga_id: entry.id.clone(), source_id: Source::MangaDex as i32, source_manga_id: m.id.clone(), source_manga_url: format!("https://mangadex.org/title/{}", m.id) };
                        msd_map.entry(key).or_default().push(msd);
                    }
                },
                Err(e) => error!("mangadex crawl error: {}", e),
            }
            {
                let mut p = data_clone.crawl_progress.lock().unwrap();
                p.sources.push(SourceProgress { name: "MangaDex".to_string(), fetched_manga: manga_map.len(), inserted_msd: msd_map.values().map(|v| v.len()).sum() });
            }
        }

        // Small pause between sources
        sleep(Duration::from_millis(300)).await;

        // Kagane best-effort
        if allowed(Source::Kagane as i32) {
            {
                let mut p = data_clone.crawl_progress.lock().unwrap();
                p.current_source = Some("Kagane".to_string());
            }
            let kagane_list = tokio::time::timeout(Duration::from_secs(15), crate::sources::kagane::search_all_series_with_urls(client)).await;
            match kagane_list {
                Ok(Ok(items)) => {
                    let mut _fetched = 0usize;
                    for (m, url) in items {
                        _fetched += 1;
                        let key = normalize_title(&m.title);
                        let entry = manga_map.entry(key.clone()).or_insert_with(|| Manga { id: uuid::Uuid::new_v4().to_string(), ..m.clone() });
                        if entry.cover_url.is_none() && m.cover_url.is_some() { entry.cover_url = m.cover_url.clone(); }
                        // Kagane source itself
                        let msd = MangaSourceData { manga_id: entry.id.clone(), source_id: Source::Kagane as i32, source_manga_id: url.clone(), source_manga_url: url.clone() };
                        msd_map.entry(key.clone()).or_default().push(msd);
                        // External providers discovered on Kagane series page
                        for (sid, link) in crate::sources::kagane::extract_provider_links(client, &url).await.into_iter() {
                            msd_map.entry(key.clone()).or_default().push(MangaSourceData { manga_id: entry.id.clone(), source_id: sid, source_manga_id: link.clone(), source_manga_url: link });
                        }
                    }
                },
                Ok(Err(e)) => error!("kagane crawl error: {}", e),
                Err(_) => error!("kagane crawl timeout"),
            }
            {
                let mut p = data_clone.crawl_progress.lock().unwrap();
                p.sources.push(SourceProgress { name: "Kagane".to_string(), fetched_manga: 0, inserted_msd: 0 });
            }
        }
        sleep(Duration::from_millis(300)).await;

        // FireScans
        if allowed(Source::FireScans as i32) {
            match crate::sources::firescans::search_manga_with_urls(client, "").await {
                Ok(items) => {
                    for (m, url) in items {
                        let key = normalize_title(&m.title);
                        let entry = manga_map.entry(key.clone()).or_insert_with(|| Manga { id: uuid::Uuid::new_v4().to_string(), ..m.clone() });
                        if entry.cover_url.is_none() && m.cover_url.is_some() { entry.cover_url = m.cover_url.clone(); }
                        let msd = MangaSourceData { manga_id: entry.id.clone(), source_id: Source::FireScans as i32, source_manga_id: url.clone(), source_manga_url: url };
                        msd_map.entry(key).or_default().push(msd);
                    }
                },
                Err(e) => error!("firescans crawl error: {}", e),
            }
        }
        sleep(Duration::from_millis(300)).await;

        // RizzComic
        if allowed(Source::RizzComic as i32) {
            match crate::sources::rizzcomic::search_manga_with_urls(client, "").await {
                Ok(items) => {
                    for (m, url) in items {
                        let key = normalize_title(&m.title);
                        let entry = manga_map.entry(key.clone()).or_insert_with(|| Manga { id: uuid::Uuid::new_v4().to_string(), ..m.clone() });
                        if entry.cover_url.is_none() && m.cover_url.is_some() { entry.cover_url = m.cover_url.clone(); }
                        let msd = MangaSourceData { manga_id: entry.id.clone(), source_id: Source::RizzComic as i32, source_manga_id: url.clone(), source_manga_url: url };
                        msd_map.entry(key).or_default().push(msd);
                    }
                },
                Err(e) => error!("rizzcomic crawl error: {}", e),
            }
        }
        sleep(Duration::from_millis(300)).await;

        // DrakeComic
        if allowed(Source::DrakeComic as i32) {
            match crate::sources::drakecomic::search_manga_with_urls(client, "").await {
                Ok(items) => {
                    for (m, url) in items {
                        let key = normalize_title(&m.title);
                        let entry = manga_map.entry(key.clone()).or_insert_with(|| Manga { id: uuid::Uuid::new_v4().to_string(), ..m.clone() });
                        if entry.cover_url.is_none() && m.cover_url.is_some() { entry.cover_url = m.cover_url.clone(); }
                        let msd = MangaSourceData { manga_id: entry.id.clone(), source_id: Source::DrakeComic as i32, source_manga_id: url.clone(), source_manga_url: url };
                        msd_map.entry(key).or_default().push(msd);
                    }
                },
                Err(e) => error!("drakecomic crawl error: {}", e),
            }
        }
        sleep(Duration::from_millis(300)).await;

        // Asmotoon
        if allowed(Source::Asmotoon as i32) {
            match crate::sources::asmotoon::search_manga_with_urls(client, "").await {
                Ok(items) => {
                    for (m, url) in items {
                        let key = normalize_title(&m.title);
                        let entry = manga_map.entry(key.clone()).or_insert_with(|| Manga { id: uuid::Uuid::new_v4().to_string(), ..m.clone() });
                        if entry.cover_url.is_none() && m.cover_url.is_some() { entry.cover_url = m.cover_url.clone(); }
                        let msd = MangaSourceData { manga_id: entry.id.clone(), source_id: Source::Asmotoon as i32, source_manga_id: url.clone(), source_manga_url: url };
                        msd_map.entry(key).or_default().push(msd);
                    }
                },
                Err(e) => error!("asmotoon crawl error: {}", e),
            }
        }
        sleep(Duration::from_millis(300)).await;

        // ResetScans
        if allowed(Source::ResetScans as i32) {
            match crate::sources::reset_scans::search_manga_with_urls(client, "").await {
                Ok(items) => {
                    for (m, url) in items {
                        let key = normalize_title(&m.title);
                        let entry = manga_map.entry(key.clone()).or_insert_with(|| Manga { id: uuid::Uuid::new_v4().to_string(), ..m.clone() });
                        if entry.cover_url.is_none() && m.cover_url.is_some() { entry.cover_url = m.cover_url.clone(); }
                        let msd = MangaSourceData { manga_id: entry.id.clone(), source_id: Source::ResetScans as i32, source_manga_id: url.clone(), source_manga_url: url };
                        msd_map.entry(key).or_default().push(msd);
                    }
                },
                Err(e) => error!("reset-scans crawl error: {}", e),
            }
        }
        sleep(Duration::from_millis(300)).await;

        // TempleScan
        if allowed(Source::TempleScan as i32) {
            match crate::sources::temple_scan::search_manga_with_urls(client, "").await {
                Ok(items) => {
                    for (m, url) in items {
                        let key = normalize_title(&m.title);
                        let entry = manga_map.entry(key.clone()).or_insert_with(|| Manga { id: uuid::Uuid::new_v4().to_string(), ..m.clone() });
                        if entry.cover_url.is_none() && m.cover_url.is_some() { entry.cover_url = m.cover_url.clone(); }
                        let msd = MangaSourceData { manga_id: entry.id.clone(), source_id: Source::TempleScan as i32, source_manga_id: url.clone(), source_manga_url: url };
                        msd_map.entry(key).or_default().push(msd);
                    }
                },
                Err(e) => error!("temple-scan crawl error: {}", e),
            }
        }
        sleep(Duration::from_millis(300)).await;

        // ThunderScans
        if allowed(Source::ThunderScans as i32) {
            match crate::sources::thunderscans::search_manga_with_urls(client, "").await {
                Ok(items) => {
                    for (m, url) in items {
                        let key = normalize_title(&m.title);
                        let entry = manga_map.entry(key.clone()).or_insert_with(|| Manga { id: uuid::Uuid::new_v4().to_string(), ..m.clone() });
                        if entry.cover_url.is_none() && m.cover_url.is_some() { entry.cover_url = m.cover_url.clone(); }
                        let msd = MangaSourceData { manga_id: entry.id.clone(), source_id: Source::ThunderScans as i32, source_manga_id: url.clone(), source_manga_url: url };
                        msd_map.entry(key).or_default().push(msd);
                    }
                },
                Err(e) => error!("thunderscans crawl error: {}", e),
            }
        }
        // WP-Manga sources (dedicated wrappers)
        // Asurascans
        if allowed(11) {
            match crate::sources::asurascans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:11, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("asurascans crawl error: {}", e),
            }
        }
        if allowed(25) {
            match crate::sources::kenscans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:25, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("kenscans crawl error: {}", e),
            }
        }
        if allowed(43) {
            match crate::sources::sirenscans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:43, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("sirenscans crawl error: {}", e),
            }
        }
        if allowed(56) {
            match crate::sources::vortexscans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:56, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("vortexscans crawl error: {}", e),
            }
        }
        if allowed(59) {
            match crate::sources::witchscans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:59, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("witchscans crawl error: {}", e),
            }
        }
        if allowed(38) {
            match crate::sources::qiscans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:38, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("qiscans crawl error: {}", e),
            }
        }
        if allowed(30) {
            match crate::sources::madarascans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:30, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("madarascans crawl error: {}", e),
            }
        }
        if allowed(39) {
            match crate::sources::rizzfables::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:39, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("rizzfables crawl error: {}", e),
            }
        }
        if allowed(40) {
            match crate::sources::rokaricomics::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:40, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("rokaricomics crawl error: {}", e),
            }
        }
        if allowed(45) {
            match crate::sources::stonescape::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:45, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("stonescape crawl error: {}", e),
            }
        }
        if allowed(31) {
            match crate::sources::manhuaus::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:31, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("manhuaus crawl error: {}", e),
            }
        }
        if allowed(19) {
            match crate::sources::grimscans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:19, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("grimscans crawl error: {}", e),
            }
        }
        if allowed(20) {
            match crate::sources::hivetoons::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:20, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("hivetoons crawl error: {}", e),
            }
        }
        if allowed(34) {
            match crate::sources::nyxscans::search_manga_with_urls(client, "").await {
                Ok(items) => for (m,url) in items { let key=normalize_title(&m.title); let entry=manga_map.entry(key.clone()).or_insert_with(|| Manga{ id: uuid::Uuid::new_v4().to_string(), ..m.clone() }); if entry.cover_url.is_none() && m.cover_url.is_some(){ entry.cover_url=m.cover_url.clone(); } msd_map.entry(key).or_default().push(MangaSourceData{ manga_id: entry.id.clone(), source_id:34, source_manga_id:url.clone(), source_manga_url:url}); },
                Err(e) => error!("nyxscans crawl error: {}", e),
            }
        }

        for (key, m) in manga_map.iter() {
            if let Err(e) = db::insert_manga(&tx, m) { error!("insert manga {}: {}", m.title, e); }
            if let Some(msds) = msd_map.get(key) {
                for msd in msds {
                    let msd_id = match db::insert_manga_source_data(&tx, msd) { Ok(id) => id, Err(e) => { error!("insert msd: {}", e); continue; } };
                    let chapters: Vec<Chapter> = if msd.source_id == Source::MangaDex as i32 {
                        match crate::sources::mangadex::get_chapters(client, &msd.source_manga_id).await { Ok(c) => c, Err(_) => Vec::new() }
                    } else if msd.source_id == Source::Kagane as i32 {
                        match crate::sources::kagane::get_chapters(client, &msd.source_manga_url).await { Ok(c) => c, Err(_) => Vec::new() }
                    } else if msd.source_id == Source::FireScans as i32 {
                        match crate::sources::firescans::get_chapters(client, &msd.source_manga_url).await { Ok(c) => c, Err(_) => Vec::new() }
                    } else if msd.source_id == Source::RizzComic as i32 {
                        match crate::sources::rizzcomic::get_chapters(client, &msd.source_manga_url).await { Ok(c) => c, Err(_) => Vec::new() }
                    } else if msd.source_id == Source::DrakeComic as i32 {
                        match crate::sources::drakecomic::get_chapters(client, &msd.source_manga_url).await { Ok(c) => c, Err(_) => Vec::new() }
                    } else if msd.source_id == Source::Asmotoon as i32 {
                        match crate::sources::asmotoon::get_chapters(client, &msd.source_manga_url).await { Ok(c) => c, Err(_) => Vec::new() }
                    } else if msd.source_id == Source::ResetScans as i32 {
                        match crate::sources::reset_scans::get_chapters(client, &msd.source_manga_url).await { Ok(c) => c, Err(_) => Vec::new() }
                    } else if msd.source_id == Source::TempleScan as i32 {
                        match crate::sources::temple_scan::get_chapters(client, &msd.source_manga_url).await { Ok(c) => c, Err(_) => Vec::new() }
                    } else if msd.source_id == Source::ThunderScans as i32 {
                        match crate::sources::thunderscans::get_chapters(client, &msd.source_manga_url).await { Ok(c) => c, Err(_) => Vec::new() }
                    } else {
                        // Generic WP-Manga sources by id
                        match msd.source_id {
                            11 => match crate::sources::wp_manga::get_chapters_base(client, "https://asurascans.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            25 => match crate::sources::wp_manga::get_chapters_base(client, "https://kenscans.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            43 => match crate::sources::wp_manga::get_chapters_base(client, "https://sirenscans.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            56 => match crate::sources::wp_manga::get_chapters_base(client, "https://vortexscans.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            59 => match crate::sources::wp_manga::get_chapters_base(client, "https://witchscans.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            38 => match crate::sources::wp_manga::get_chapters_base(client, "https://qiscans.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            30 => match crate::sources::wp_manga::get_chapters_base(client, "https://madarascans.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            39 => match crate::sources::wp_manga::get_chapters_base(client, "https://rizzfables.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            40 => match crate::sources::wp_manga::get_chapters_base(client, "https://rokaricomics.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            45 => match crate::sources::wp_manga::get_chapters_base(client, "https://stonescape.xyz", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            31 => match crate::sources::wp_manga::get_chapters_base(client, "https://manhuaus.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            19 => match crate::sources::wp_manga::get_chapters_base(client, "https://grimscans.team", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            20 => match crate::sources::wp_manga::get_chapters_base(client, "https://hivetoons.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            34 => match crate::sources::wp_manga::get_chapters_base(client, "https://nyxscans.com", &msd.source_manga_url).await { Ok(c)=>c, Err(_)=>Vec::new() },
                            _ => Vec::new(),
                        }
                    };
                    let _ = db::insert_chapters(&tx, msd_id, &chapters);
                }
            }
        }

        if let Err(e) = tx.commit() { error!("commit error: {}", e); }
        info!("Full crawl finished.");
        {
            let mut p = data_clone.crawl_progress.lock().unwrap();
            p.in_progress = false;
            p.finished_at = Some(Utc::now().timestamp());
            p.current_source = None;
        }
    });
}

pub fn get_progress(data: web::Data<crate::AppState>) -> CrawlProgress {
    data.crawl_progress.lock().unwrap().clone()
}
