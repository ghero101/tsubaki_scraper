# Tsubaki Scraper Architecture

## Overview

Tsubaki Scraper is a comprehensive manga aggregation and scraping system built in Rust. It supports 90+ manga sources with sophisticated bot detection bypass, browser automation, and metadata aggregation capabilities.

## Project Structure

```
tsubaki_scraper/
├── src/
│   ├── main.rs                 # HTTP API server (Actix-web)
│   ├── lib.rs                  # Library interface for external use
│   │
│   ├── Core Modules
│   ├── models.rs               # Data structures (Manga, Chapter, Source enums)
│   ├── config.rs               # Configuration management
│   ├── app_state.rs            # Application state for Actix-web
│   ├── helpers.rs              # Utility functions (parsing, normalization, XML)
│   │
│   ├── HTTP & Browser
│   ├── http_client.rs          # Enhanced HTTP client with bot detection bypass
│   ├── browser_client.rs       # Headless Chrome wrapper
│   ├── cloudflare_bypass.rs    # Anti-bot detection strategies
│   ├── browser/                # Browser automation module
│   │   ├── mod.rs
│   │   ├── config.rs           # Browser configuration
│   │   ├── manager.rs          # Tab and session management
│   │   └── scraper.rs          # HTML extraction with browser
│   │
│   ├── Data Layer
│   ├── db.rs                   # SQLite database operations
│   ├── scraper.rs              # Chapter download & ZIP creation
│   ├── crawler.rs              # Manga discovery and monitoring
│   ├── scheduler.rs            # Background task scheduling
│   ├── metrics.rs              # Performance tracking
│   │
│   ├── Metadata
│   ├── metadata/
│   │   ├── mod.rs
│   │   ├── aggregate.rs        # Multi-source metadata aggregation
│   │   ├── anilist.rs          # AniList API integration
│   │   ├── mal.rs              # MyAnimeList API
│   │   └── mangabaka.rs        # Mangabaka metadata
│   │
│   └── sources/                # 90+ source implementations
│       ├── mod.rs              # Source registry
│       ├── mangadex.rs         # MangaDex API
│       ├── wp_manga.rs         # Base WP-Manga implementation
│       ├── *_browser.rs        # Browser-based scrapers (25+)
│       └── [60+ other sources]
│
├── tests/                       # Integration tests
│   ├── browser_client_tests.rs
│   ├── http_client_tests.rs
│   ├── source_validation_test.rs
│   ├── chapter_download_test.rs
│   └── end_to_end_tests.rs
│
├── examples/                    # Example programs (21+)
│   └── test_*.rs
│
└── Configuration
    ├── Cargo.toml
    ├── cloudflare_config.toml
    └── log4rs.yml
```

## Core Components

### 1. HTTP API Server (main.rs)

The main entry point is an Actix-web HTTP server providing RESTful endpoints:

#### Manga Endpoints
- `GET /manga` - List/search manga
- `GET /manga/{id}` - Get manga details with all sources
- `POST /manga/{id}/monitor` - Start monitoring for new chapters
- `GET /manga/{id}/chapters` - Get all chapters across sources

#### Source Endpoints
- `GET /sources` - List available sources
- `GET /sources/{source_id}/manga` - Get manga from specific source

#### Import Endpoints
- `GET /import` - Import all sources
- `GET /import/source/{source}` - Import specific source
- `GET /import/source/{source}/manga` - Import manga only (no chapters)

#### Download Endpoints
- `GET /download/{manga_id}/{chapter_number}` - Download chapter
- `GET /download/{manga_id}/{chapter_number}/{source_id}` - Download from specific source
- `GET /download/byurl` - Download by direct URL

#### Stats & Metrics
- `GET /stats` - Server statistics
- `GET /metrics` - Source metrics
- `GET /metrics/summary` - Metrics summary

### 2. Data Models (models.rs)

```rust
pub enum Source {
    MangaDex = 1,
    FireScans = 2,
    RizzComic = 3,
    // ... 50+ more sources
}

pub struct Manga {
    pub id: String,
    pub title: String,
    pub alt_titles: Option<String>,
    pub cover_url: Option<String>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub rating: Option<String>,
    pub monitored: Option<bool>,
}

pub struct Chapter {
    pub id: i32,
    pub manga_source_data_id: i32,
    pub chapter_number: String,
    pub url: String,
    pub scraped: bool,
}

pub struct MangaSourceData {
    pub manga_id: String,
    pub source_id: i32,
    pub source_manga_id: String,
    pub source_manga_url: String,
}
```

### 3. Bot Detection Bypass

#### HTTP Client Strategy (http_client.rs)
- Custom user-agent rotation
- Cookie persistence
- Retry logic with exponential backoff
- Rate limiting
- Header spoofing
- Compression support (gzip, brotli)

#### Browser Automation (browser_client.rs)
- Headless Chrome via chromiumoxide
- JavaScript execution
- Cloudflare challenge detection
- Stealth mode (hides webdriver property)
- Configurable timeouts and window sizes
- Image loading control

#### Cloudflare Bypass (cloudflare_bypass.rs)
- TLS fingerprinting
- JA3 signature randomization
- Header normalization
- CAPTCHA solver integration (2Captcha, AntiCaptcha)

### 4. Source Architecture

Sources are categorized into:

#### HTTP-Based Sources (40+)
- Direct HTTP requests with enhanced client
- HTML parsing with scraper crate
- Examples: MangaDex, FireScans, RizzComic

#### Browser-Based Sources (25+)
- Headless Chrome for JavaScript-heavy sites
- Cloudflare-protected sites
- Next.js/React applications
- Examples: *_browser.rs files

#### WP-Manga Sources
- WordPress Madara theme sites
- Common pattern: `wp_manga.rs` base implementation
- Per-site wrappers: AsuraScans, KenScans, QIScans, etc.

### 5. Database Schema (SQLite)

```sql
-- Sources registry
CREATE TABLE sources (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL
);

-- Manga metadata
CREATE TABLE manga (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    alt_titles TEXT,
    cover_url TEXT,
    description TEXT,
    tags TEXT,
    rating TEXT,
    monitored BOOLEAN,
    check_interval_secs INTEGER,
    last_chapter_check INTEGER
);

-- Source-specific manga data
CREATE TABLE manga_source_data (
    id INTEGER PRIMARY KEY,
    manga_id TEXT NOT NULL,
    source_id INTEGER NOT NULL,
    source_manga_id TEXT NOT NULL,
    source_manga_url TEXT NOT NULL,
    FOREIGN KEY (manga_id) REFERENCES manga(id),
    FOREIGN KEY (source_id) REFERENCES sources(id)
);

-- Chapters per manga per source
CREATE TABLE chapters (
    id INTEGER PRIMARY KEY,
    manga_source_data_id INTEGER NOT NULL,
    chapter_number TEXT NOT NULL,
    url TEXT NOT NULL,
    scraped BOOLEAN DEFAULT 0,
    FOREIGN KEY (manga_source_data_id) REFERENCES manga_source_data(id)
);

-- External provider IDs
CREATE TABLE provider_ids (
    id INTEGER PRIMARY KEY,
    manga_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    UNIQUE(manga_id, provider),
    FOREIGN KEY (manga_id) REFERENCES manga(id)
);
```

### 6. Background Services

#### Scheduler (scheduler.rs)
- Runs every 60 seconds
- Checks for manga due for chapter updates
- Fetches new chapters from all sources
- Updates database

#### Crawler (crawler.rs)
- Full source crawl for manga discovery
- Batch processing with progress tracking
- Configurable source filtering
- Merge strategy for duplicate manga

#### Metadata Aggregation (metadata/)
- Fetch from AniList GraphQL API
- Fetch from MyAnimeList REST API
- Merge metadata from multiple sources
- Automatic matching by title normalization

## Configuration

### Bot Detection Config (config.toml)
```toml
[bot_detection]
enable_enhanced_client = true
enable_browser = true
max_retries = 3
timeout_secs = 30
initial_retry_delay_ms = 1000
retry_delay_multiplier = 2.0
enable_cookies = true
enable_compression = true
browser_timeout_secs = 45
browser_headless = true
browser_disable_images = true
rate_limit_delay_ms = 500
```

### Cloudflare Bypass Config (cloudflare_config.toml)
```toml
[cloudflare]
enable_ja3_randomization = true
enable_tls_fingerprint_spoofing = true
enable_header_normalization = true

[captcha]
provider = "2captcha"  # or "anticaptcha"
api_key = "your_key_here"
```

## Testing Strategy

### Unit Tests
- Embedded in source files with `#[cfg(test)]`
- Test parsing logic, HTML extraction
- 13 source files with unit tests

### Integration Tests (tests/)
- `source_validation_test.rs` - Validates all 61 sources
- `chapter_download_test.rs` - Tests actual chapter downloads
- `browser_client_tests.rs` - Browser automation tests
- `http_client_tests.rs` - HTTP client tests
- `end_to_end_tests.rs` - Complete workflows

### Example Programs (examples/)
- 21+ runnable examples
- Per-source testing programs
- Browser vs HTTP comparisons

## Performance Metrics

The `MetricsTracker` records:
- Request latency (avg, min, max)
- Success/failure rates
- Rate limit detection
- Cloudflare challenge counts
- Timeout tracking
- Per-source statistics

## Build & Run

```bash
# Build
cargo build --release

# Run server
cargo run --release

# Run tests (fast, HTTP only)
MANGA_SCRAPER_USE_BROWSER=0 cargo test -- --nocapture

# Run tests (full, with browser)
cargo test -- --ignored --nocapture

# Run specific example
cargo run --example test_sources
```

## Dependencies

### Core
- `actix-web` - HTTP server
- `reqwest` - HTTP client
- `rusqlite` - SQLite database
- `tokio` - Async runtime

### Scraping
- `scraper` - HTML parsing (CSS selectors)
- `headless_chrome` / `chromiumoxide` - Browser automation
- `serde` / `serde_json` - JSON serialization

### Utilities
- `uuid` - Unique ID generation
- `chrono` - Date/time handling
- `regex` - Pattern matching
- `log` / `log4rs` - Logging

## Development Guidelines

1. **Adding a New Source**:
   - Create `src/sources/newsource.rs`
   - Implement `search_manga_with_urls()` and `get_chapters()`
   - Add to `src/sources/mod.rs`
   - Add enum variant to `models.rs::Source`
   - Create test in `examples/test_newsource.rs`

2. **Bot Detection Issues**:
   - Try enhanced HTTP client first (`http_client.rs`)
   - If blocked, use browser automation (`*_browser.rs`)
   - Configure stealth mode in browser config
   - Adjust rate limits and retries

3. **Testing**:
   - Write unit tests for parsing logic
   - Add integration test case
   - Test both HTTP and browser methods
   - Verify chapter download works

## Architecture Decisions

### Why SQLite?
- Embedded, no separate server
- Good for single-instance deployments
- ACID compliance for data integrity
- Easy backup (single file)

### Why Actix-web?
- High performance (Techempower benchmarks)
- Mature ecosystem
- Good async support with Tokio

### Why Both HTTP and Browser?
- HTTP is faster and more efficient
- Browser handles JavaScript-rendered content
- Automatic fallback strategy
- Different sources need different approaches

### Why Rust?
- Memory safety without garbage collection
- Excellent async/await support
- Strong type system prevents bugs
- Fast compiled binaries

## Future Improvements

- [ ] Add GraphQL API alongside REST
- [ ] Implement caching layer (Redis)
- [ ] Add user authentication
- [ ] Create web UI
- [ ] Docker containerization
- [ ] Kubernetes deployment configs
- [ ] More comprehensive metrics/observability
- [ ] Rate limiting per-user
- [ ] Webhooks for new chapter notifications
