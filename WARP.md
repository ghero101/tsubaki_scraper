# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Common commands

- Build: `cargo build`
- Run: `cargo run`
- Format (write): `cargo fmt --all`
- Format (check CI-friendly): `cargo fmt --all -- --check`
- Lint: `cargo clippy --all-targets --all-features -- -D warnings`
- Test (all): `cargo test`
- Test (single): `cargo test <module_path>::<test_name> -- --nocapture`
- Clean build: `cargo clean && cargo build`

## Run the API locally

```bash path=null start=null
cargo run
# Server auto-binds to first available port 8080-8090
```

### Legacy import (deprecated, use crawl endpoints instead):
```bash path=null start=null
curl -X GET http://127.0.0.1:8080/import
```

### Import by source:
```bash path=null start=null
# Import all manga + chapters from a single source
curl -X GET http://127.0.0.1:8080/import/source/mangadex
curl -X GET http://127.0.0.1:8080/import/source/firescans
curl -X GET http://127.0.0.1:8080/import/source/asurascans
# Manga only (no chapters)
curl -X GET http://127.0.0.1:8080/import/source/firescans/manga
# Quick import (first page only, limited items)
curl -X GET "http://127.0.0.1:8080/import/source/firescans/quick?limit=10&chapters=1"
```

### Full crawl (recommended):
```bash path=null start=null
# Start async crawl (excludes Kagane by default)
curl -X GET http://127.0.0.1:8080/crawl/full
# With filters
curl -X GET "http://127.0.0.1:8080/crawl/full?include=mangadex,firescans"
curl -X GET "http://127.0.0.1:8080/crawl/full?exclude=kagane"
# Check progress
curl http://127.0.0.1:8080/crawl/status
```

List manga (pagination, search, sorting, rating filter supported):
```bash path=null start=null
curl "http://127.0.0.1:8080/manga?limit=10&offset=0&sort=title&search=solo&tags=action&rating=safe"
```

Get one manga by id, then its chapters:
```bash path=null start=null
curl http://127.0.0.1:8080/manga/<ID>
curl http://127.0.0.1:8080/manga/<ID>/chapters
```

Enable monitoring for a manga and set intervals (seconds):
```bash path=null start=null
curl -X POST http://127.0.0.1:8080/manga/<ID>/monitor \
  -H "Content-Type: application/json" \
  -d '{"monitored": true, "check_interval_secs": 3600, "discover_interval_secs": 86400}'
```

Download a chapter (stream to client) or save to disk on the server:
```bash path=null start=null
# Stream a CBZ
curl -L "http://127.0.0.1:8080/download/<MANGA_ID>/<CHAPTER_NUMBER>?stream=true" -o chapter.cbz
# Save on server (in downloads dir configured below)
curl -L "http://127.0.0.1:8080/download/<MANGA_ID>/<CHAPTER_NUMBER>"
# If multiple sources exist, pick one explicitly
curl -L "http://127.0.0.1:8080/download/<MANGA_ID>/<CHAPTER_NUMBER>/<SOURCE_ID>?stream=true" -o chapter.cbz
```

Download layout on disk:
- Root: `downloads/<Manga Title>/`
  - Chapters: `<sourceId>-<SourceName>-<Manga Title> - <Vol/Ch label>.cbz`
  - Covers: `downloads/<Manga Title>/covers/` (auto-downloaded on first save)
  - Artwork: `downloads/<Manga Title>/artwork/` (reserved for future use)

Metadata providers (MangaBaka, MAL, AniList):
```bash path=null start=null
# Sync individual providers (async, returns 202 Accepted)
curl -X POST http://127.0.0.1:8080/metadata/mangabaka/sync
curl -X POST http://127.0.0.1:8080/metadata/mal/sync
curl -X POST http://127.0.0.1:8080/metadata/anilist/sync
# With limits
curl -X POST "http://127.0.0.1:8080/metadata/mangabaka/sync?limit=100"
# Aggregate metadata from all providers into manga fields
curl -X POST http://127.0.0.1:8080/metadata/aggregate/sync
# Check progress
curl http://127.0.0.1:8080/metadata/status
# Cancel running metadata sync
curl -X POST http://127.0.0.1:8080/metadata/cancel
```

Stats and verification:
```bash path=null start=null
# Get overall stats and per-source breakdown
curl http://127.0.0.1:8080/stats
# Verify downloads from each source (test scraping)
curl http://127.0.0.1:8080/verify/downloads
```

Alternative download by direct URL:
```bash path=null start=null
curl -X GET "http://127.0.0.1:8080/download/byurl?manga_id=<ID>&url=<CHAPTER_URL>&stream=true" -o chapter.cbz
```

## Configuration and logging

- Config file: `config.toml` (optional). Currently supports:
  - `download_dir = "downloads"` (default if file is absent or invalid)
- Logging: `log4rs.yml` controls logging; logs are written per that config (a sample `log/app.log` is present).
- Database: `manga.db` (SQLite, auto-created on first run)

## High-level architecture

- Web/API layer (Actix Web)
  - `src/main.rs` wires routes and shared `AppState`:
    - SQLite connection (rusqlite) guarded by `Mutex`
    - Shared `reqwest::Client`
    - Loaded `Config`
    - Crawl progress tracker (`Mutex<CrawlProgress>`)
    - Metadata sync progress tracker (`Mutex<MetadataProgress>`)
    - Metadata cancel flag (`Mutex<bool>`)
  - Endpoints:
    - **Legacy**: `GET /import` - aggregates from all sources (use crawl endpoints instead)
    - **Import**: `GET /import/source/{source}` - import single source with chapters
    - **Import**: `GET /import/source/{source}/manga` - import source metadata only
    - **Import**: `GET /import/source/{source}/quick` - fast partial import (first page)
    - **Crawl**: `GET /crawl/full` - async full crawl with filters (include/exclude sources)
    - **Crawl**: `GET /crawl/status` - crawl progress info
    - **Manga**: `GET /manga` - pagination, search (title/alt_titles), optional `tags` and `rating` filters, sort by `title` or `rating`
    - **Manga**: `GET /manga/{id}` - returns manga plus source links
    - **Manga**: `GET /manga/{id}/chapters` - returns all chapters across sources
    - **Manga**: `POST /manga/{id}/monitor` - toggles monitoring and sets intervals
    - **Download**: `GET /download/{manga_id}/{chapter_number}` - stream or save CBZ
    - **Download**: `GET /download/{manga_id}/{chapter_number}/{source_id}` - download from specific source
    - **Download**: `GET /download/byurl` - download by direct chapter URL
    - **Metadata**: `POST /metadata/{provider}/sync` - async metadata sync (MangaBaka, MAL, AniList)
    - **Metadata**: `POST /metadata/aggregate/sync` - merge metadata into manga records
    - **Metadata**: `GET /metadata/status` - metadata sync progress
    - **Metadata**: `POST /metadata/cancel` - cancel running metadata sync
    - **Stats**: `GET /stats` - returns aggregate counts and per-source breakdown
    - **Verify**: `GET /verify/downloads` - test download from each source

- Data layer (SQLite via rusqlite)
  - `src/db.rs` owns schema creation/migrations, seed of `sources`, and all queries/updates:
    - Tables: `sources`, `manga`, `manga_source_data`, `chapters`, `provider_ids` with indexes
    - `manga` now has: `mangabaka_id`, `mal_id`, `anilist_id` columns for metadata provider IDs
    - Upserts manga, ensures uniqueness of `(manga_id, source_id)` and `(manga_source_data_id, url)`
    - Pagination/search SQL for `GET /manga`; due checks for scheduler; chapter retrieval per source
    - Per-source stats queries

- Domain models
  - `src/models.rs` defines:
    - `Manga`: core manga model with metadata fields, monitoring config
    - `Chapter`, `MangaSourceData`: chapter and source linkage
    - `Source` enum: MangaDex (1), FireScans (2), RizzComic (3), MyAnimeList (4), AniList (5), DrakeComic (6), KDTNovels (7), Asmotoon (8), ResetScans (9), Kagane (10), TempleScan (49), ThunderScans (50)
    - Pagination structs, request/response DTOs (`MangaWithSources`, `ChapterWithSource`, `MonitorRequest`, `Stats`)

- Scrapers and sources
  - `src/sources/*` provides per-site adapters:
    - **MangaDex**: `mangadex.rs` (REST API for search/feed, At-Home for downloads)
    - **Direct implementations**: `firescans.rs`, `rizzcomic.rs`, `drakecomic.rs`, `asmotoon.rs`, `reset_scans.rs`, `kagane.rs`, `temple_scan.rs`, `thunderscans.rs` (HTML scraping)
    - **WP-Manga wrappers** (via `wp_manga.rs`): `asurascans.rs` (11), `kenscans.rs` (25), `sirenscans.rs` (43), `vortexscans.rs` (56), `witchscans.rs` (59), `qiscans.rs` (38), `madarascans.rs` (30), `rizzfables.rs` (39), `rokaricomics.rs` (40), `stonescape.rs` (45), `manhuaus.rs` (31), `grimscans.rs` (19), `hivetoons.rs` (20), `nyxscans.rs` (34)
    - **Metadata-only**: `kdtnovels.rs` (7) - no chapter scraping
  - Each exposes `search_manga_with_urls`, `get_chapters`, and optionally `search_manga_first_page` for quick imports.
  - Source IDs in parentheses indicate database ID assignment for WP-Manga sites.

- Download pipeline
  - `src/scraper.rs` builds CBZ archives:
    - MangaDex: uses At-Home API to fetch page images
    - Other sites: scrapes `img` elements from reading pages
    - Writes to in-memory buffer for streaming or to disk under `Config.download_dir` (filenames sanitized)
    - Embeds ComicInfo.xml metadata in CBZ files (series, number, summary, genres)
    - Auto-downloads cover images on first chapter save

- Background scheduler
  - `src/scheduler.rs` runs every 60s, finds monitored manga due for checks, refreshes chapters via the appropriate source scraper, and persists new chapters; updates `last_chapter_check` timestamps.
  - Spawned automatically on server start.

- Async crawling system
  - `src/crawler.rs` implements full-site crawling:
    - Spawns async background task for multi-source crawl
    - Progress tracking with `CrawlProgress` (per-source stats, timestamps, errors)
    - Source filtering (include/exclude by ID or name)
    - Default excludes Kagane (slow/unreliable) unless explicitly included
    - Normalizes titles for de-duplication across sources

- Metadata providers
  - `src/metadata/mangabaka.rs` - MangaBaka API integration for genres/descriptions
  - `src/metadata/mal.rs` - MyAnimeList API integration for metadata and ratings
  - `src/metadata/anilist.rs` - AniList GraphQL API for metadata and adult flags
  - `src/metadata/aggregate.rs` - merges metadata from all providers into manga records
  - All run async with progress tracking and cancellation support

- Configuration
  - `src/config.rs` loads `config.toml` if present, otherwise defaults; currently only `download_dir` is used.

## Key features

- **Multi-source aggregation**: Merges manga from 30+ sources by normalized title
- **Async crawling**: Non-blocking full-site imports with progress tracking
- **Metadata enrichment**: Integrates MangaBaka, MAL, AniList for descriptions, genres, ratings
- **Flexible downloads**: Stream CBZ directly or save to disk with ComicInfo.xml
- **Monitoring**: Auto-checks for new chapters on configured intervals
- **Smart chapter matching**: Fuzzy matching for chapter numbers (exact, normalized, numeric-only, substring)
- **Port auto-bind**: Tries ports 8080-8090 to avoid conflicts
- **Progress APIs**: Real-time status for crawls and metadata syncs
