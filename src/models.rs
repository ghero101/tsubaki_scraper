use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum Source {
    MangaDex = 1,
    FireScans = 2,
    RizzComic = 3,
    MyAnimeList = 4,
    AniList = 5,
    DrakeComic = 6,
    KDTNovels = 7,
    Asmotoon = 8,
    ResetScans = 9,
    Kagane = 10,
    TempleScan = 49,
    ThunderScans = 50,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manga {
    pub id: String,
    pub title: String,
    pub alt_titles: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub rating: Option<String>,
    // Monitoring fields
    pub monitored: Option<bool>,
    pub check_interval_secs: Option<i64>,
    pub discover_interval_secs: Option<i64>,
    pub last_chapter_check: Option<i64>,
    pub last_discover_check: Option<i64>,
    // pub mal_id: Option<i32>,
    // pub anilist_id: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Chapter {
    pub id: i32,
    pub manga_source_data_id: i32,
    pub chapter_number: String,
    pub url: String,
    pub scraped: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MangaSourceData {
    pub manga_id: String,
    pub source_id: i32,
    pub source_manga_id: String,
    pub source_manga_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub total: i32,
    pub limit: i32,
    pub offset: i32,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub total_manga: i32,
    pub total_chapters: i32,
    pub total_sources: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MangaWithSources {
    pub id: String,
    pub title: String,
    pub alt_titles: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub rating: Option<String>,
    pub sources: Vec<SourceInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceInfo {
    pub source_id: i32,
    pub source_name: String,
    pub source_manga_id: String,
    pub source_manga_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChapterWithSource {
    pub id: i32,
    pub chapter_number: String,
    pub url: String,
    pub scraped: bool,
    pub source_id: i32,
    pub source_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorRequest {
    pub monitored: bool,
    pub check_interval_secs: Option<i64>,
    pub discover_interval_secs: Option<i64>,
}
