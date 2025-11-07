# Browser Automation Implementation Plan

## Executive Summary

**Goal:** Implement browser automation to unlock 11+ sources (~1000+ chapters)

**Impact:**
- 7 Next.js sources: Kagane, HiveToons, KenScans, QIScans, MavinTranslations, Asmotoon, NyxScans
- 4+ anti-bot protected: DrakeComic, MadaraScans, Webtoon, Tapas
- Estimated total: **~1000+ chapters across 11+ sources**

## Technology Stack Analysis

### Option 1: `headless_chrome` ⭐ RECOMMENDED
**Crate:** `headless_chrome` (https://crates.io/crates/headless_chrome)

**Pros:**
- ✅ Pure Rust implementation
- ✅ No external dependencies (bundles Chrome DevTools Protocol)
- ✅ Good documentation and examples
- ✅ Active maintenance
- ✅ Easy setup - just needs Chrome/Chromium installed

**Cons:**
- ⚠️ Requires Chrome/Chromium on system
- ⚠️ Slightly higher memory usage

**Example Usage:**
```rust
use headless_chrome::{Browser, LaunchOptions};

let browser = Browser::new(LaunchOptions::default())?;
let tab = browser.new_tab()?;
tab.navigate_to("https://example.com")?;
tab.wait_for_element("div.content")?;
let html = tab.get_content()?;
```

### Option 2: `fantoccini`
**Crate:** `fantoccini` (WebDriver client)

**Pros:**
- ✅ WebDriver standard (works with multiple browsers)
- ✅ Good async support

**Cons:**
- ❌ Requires separate WebDriver server (geckodriver, chromedriver)
- ❌ More complex setup
- ❌ Extra process management

### Option 3: `thirtyfour`
**Crate:** `thirtyfour` (WebDriver client)

**Pros:**
- ✅ Modern async WebDriver
- ✅ Good API design

**Cons:**
- ❌ Requires separate WebDriver server
- ❌ More dependencies

**Decision: Use `headless_chrome`** - Best balance of simplicity and power

---

## Architecture Design

### Module Structure

```
rust_manga_scraper/
├── src/
│   ├── sources/
│   │   ├── hivetoons.rs (updated to use browser)
│   │   ├── kenscans.rs (updated to use browser)
│   │   └── ... (other sources)
│   ├── browser/
│   │   ├── mod.rs (main browser module)
│   │   ├── manager.rs (browser instance management)
│   │   ├── scraper.rs (browser-based scraping utilities)
│   │   └── config.rs (browser configuration)
│   └── lib.rs (export browser module)
```

### Core Components

#### 1. Browser Manager
```rust
pub struct BrowserManager {
    browser: Arc<Browser>,
    max_tabs: usize,
    timeout: Duration,
}

impl BrowserManager {
    pub fn new(config: BrowserConfig) -> Result<Self>;
    pub fn new_tab(&self) -> Result<Arc<Tab>>;
    pub fn shutdown(&self);
}
```

#### 2. Browser Scraper
```rust
pub struct BrowserScraper {
    tab: Arc<Tab>,
}

impl BrowserScraper {
    pub async fn navigate(&self, url: &str) -> Result<()>;
    pub async fn wait_for_selector(&self, selector: &str) -> Result<()>;
    pub async fn get_html(&self) -> Result<String>;
    pub async fn click(&self, selector: &str) -> Result<()>;
    pub async fn evaluate_script(&self, script: &str) -> Result<String>;
}
```

#### 3. Browser Config
```rust
pub struct BrowserConfig {
    pub headless: bool,
    pub window_size: (u32, u32),
    pub user_agent: Option<String>,
    pub timeout_seconds: u64,
    pub disable_images: bool,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            window_size: (1920, 1080),
            user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".into()),
            timeout_seconds: 30,
            disable_images: true, // Performance optimization
        }
    }
}
```

---

## Implementation Phases

### Phase 1: Setup & Basic Infrastructure (2-3 hours)

**Tasks:**
1. Add `headless_chrome` dependency to Cargo.toml
2. Create `src/browser/` module structure
3. Implement `BrowserManager` with basic initialization
4. Implement `BrowserScraper` with core navigation/scraping
5. Create unit tests for basic functionality

**Deliverables:**
- Working browser module that can navigate and extract HTML
- Basic error handling and timeout management

**Example Test:**
```rust
#[test]
fn test_browser_basic_navigation() {
    let manager = BrowserManager::new(BrowserConfig::default()).unwrap();
    let scraper = BrowserScraper::new(manager.new_tab().unwrap());

    scraper.navigate("https://example.com").await.unwrap();
    let html = scraper.get_html().await.unwrap();

    assert!(html.contains("Example Domain"));
}
```

### Phase 2: Next.js Source Integration (3-4 hours)

**Priority Order:**
1. HiveToons (already analyzed, clear pattern)
2. KenScans (already tested, verified pattern)
3. QIScans, MavinTranslations, Asmotoon, NyxScans

**Implementation Pattern:**
```rust
// src/sources/hivetoons_browser.rs
use crate::browser::{BrowserManager, BrowserScraper};

pub async fn search_manga_with_urls_browser(
    manager: &BrowserManager,
    _title: &str
) -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
    let scraper = BrowserScraper::new(manager.new_tab()?);

    // Navigate to series list
    scraper.navigate("https://hivetoons.org/series").await?;

    // Wait for content to load
    scraper.wait_for_selector("div.manga-card, a[href*='/series/']").await?;

    // Get rendered HTML
    let html = scraper.get_html().await?;

    // Parse with existing scraper logic
    let document = Html::parse_document(&html);
    // ... existing parsing logic
}

pub async fn get_chapters_browser(
    manager: &BrowserManager,
    series_url: &str
) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
    let scraper = BrowserScraper::new(manager.new_tab()?);

    scraper.navigate(series_url).await?;
    scraper.wait_for_selector("a[href*='chapter']").await?;

    let html = scraper.get_html().await?;

    // Parse chapters from rendered HTML
    let document = Html::parse_document(&html);
    // ... chapter extraction logic
}
```

**Testing Strategy:**
- Test each source individually
- Verify chapter counts improve from 1→100+
- Compare results with manual browser inspection

### Phase 3: Anti-Bot Protected Sources (2-3 hours)

**Sources:** DrakeComic, MadaraScans, Webtoon, Tapas

**Additional Requirements:**
- Stealth mode configuration
- Cookie handling
- User interaction simulation (for CAPTCHA bypass where legal)

**Stealth Configuration:**
```rust
impl BrowserConfig {
    pub fn stealth_mode() -> Self {
        Self {
            headless: true,
            user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36".into()),
            window_size: (1920, 1080),
            timeout_seconds: 60,
            disable_images: true,
            // Additional stealth settings
        }
    }
}
```

**Cloudflare Bypass:**
```rust
pub async fn wait_for_cloudflare(&self) -> Result<()> {
    // Wait for Cloudflare challenge to complete
    let start = Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(30) {
            return Err("Cloudflare challenge timeout".into());
        }

        let title = self.tab.get_title()?;
        if !title.contains("Just a moment") {
            break;
        }

        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}
```

### Phase 4: Optimization & Polish (2-3 hours)

**Performance Optimizations:**
1. **Tab pooling** - Reuse tabs instead of creating new ones
2. **Image blocking** - Disable image loading for faster parsing
3. **Script filtering** - Block unnecessary scripts (analytics, ads)
4. **Parallel scraping** - Use multiple tabs concurrently

**Example Tab Pool:**
```rust
pub struct TabPool {
    available: Arc<Mutex<Vec<Arc<Tab>>>>,
    max_size: usize,
    browser: Arc<Browser>,
}

impl TabPool {
    pub async fn acquire(&self) -> Result<Arc<Tab>> {
        let mut available = self.available.lock().await;
        if let Some(tab) = available.pop() {
            Ok(tab)
        } else {
            Ok(self.browser.new_tab()?)
        }
    }

    pub async fn release(&self, tab: Arc<Tab>) {
        let mut available = self.available.lock().await;
        if available.len() < self.max_size {
            available.push(tab);
        }
    }
}
```

**Error Handling:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("Navigation timeout: {0}")]
    NavigationTimeout(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("JavaScript error: {0}")]
    JavaScriptError(String),

    #[error("Browser error: {0}")]
    BrowserError(#[from] headless_chrome::Error),
}
```

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browser_manager_creation() {
        let config = BrowserConfig::default();
        let manager = BrowserManager::new(config);
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_navigation_and_html_extraction() {
        let manager = BrowserManager::new(BrowserConfig::default()).unwrap();
        let scraper = BrowserScraper::new(manager.new_tab().unwrap());

        scraper.navigate("https://example.com").await.unwrap();
        let html = scraper.get_html().await.unwrap();

        assert!(html.len() > 0);
        assert!(html.contains("Example"));
    }
}
```

### Integration Tests
```rust
// examples/test_hivetoons_browser.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = BrowserManager::new(BrowserConfig::default())?;

    let results = hivetoons_browser::search_manga_with_urls_browser(&manager, "").await?;
    println!("Found {} manga", results.len());

    for (manga, url) in results.iter().take(3) {
        let chapters = hivetoons_browser::get_chapters_browser(&manager, &url).await?;
        println!("{}: {} chapters", manga.title, chapters.len());
    }

    Ok(())
}
```

### Performance Benchmarks
```rust
#[bench]
fn bench_browser_vs_http(b: &mut Bencher) {
    // Compare browser scraping vs direct HTTP
    // Target: <2x slower than direct HTTP
}
```

---

## Timeline & Milestones

### Week 1: Core Infrastructure
- ✅ Day 1-2: Setup browser module, basic navigation
- ✅ Day 3-4: Wait strategies, HTML extraction
- ✅ Day 5: Error handling, timeouts

**Success Criteria:** Can navigate to any URL and extract full HTML

### Week 2: Next.js Sources
- ✅ Day 1-2: HiveToons implementation & testing
- ✅ Day 3-4: KenScans, QIScans implementation
- ✅ Day 5: Remaining Next.js sources

**Success Criteria:** All 7 Next.js sources showing 100+ chapters

### Week 3: Anti-Bot & Optimization
- ✅ Day 1-2: Anti-bot measures, Cloudflare handling
- ✅ Day 3-4: Performance optimization (tab pooling, image blocking)
- ✅ Day 5: Documentation, final testing

**Success Criteria:** 11+ sources working, documentation complete

---

## Resource Requirements

### System Requirements
- **Chrome/Chromium:** Must be installed on system
- **Memory:** ~200MB per browser instance
- **CPU:** Minimal (JavaScript execution in browser)

### Dependencies
```toml
[dependencies]
headless_chrome = "1.0"
tokio = { version = "1", features = ["full"] }
scraper = "0.17"
thiserror = "1.0"
```

### Optional Optimizations
```toml
[dependencies]
# For tab pooling
deadpool = "0.9"

# For better async coordination
async-trait = "0.1"
```

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Cloudflare blocking | High | Stealth mode, rotating user agents, delays |
| Rate limiting | Medium | Request throttling, tab pooling |
| Memory leaks | Medium | Proper tab cleanup, resource limits |
| Site structure changes | Medium | Robust selectors, fallback strategies |
| Chrome not installed | Low | Clear error messages, installation guide |

---

## Success Metrics

**Target Goals:**
- ✅ 7 Next.js sources: 1→100+ chapters each (~700 chapters)
- ✅ 4 anti-bot sources: 0→50+ chapters each (~200 chapters)
- ✅ Total improvement: **~1000+ new chapters**

**Performance Targets:**
- Browser scraping: <5s per page
- Memory usage: <500MB for 5 concurrent tabs
- Success rate: >95% for working sources

---

## Next Steps (Immediate)

1. **Add dependency:**
   ```bash
   cargo add headless_chrome
   cargo add tokio --features full
   cargo add thiserror
   ```

2. **Create module structure:**
   ```bash
   mkdir src/browser
   touch src/browser/mod.rs
   touch src/browser/manager.rs
   touch src/browser/scraper.rs
   touch src/browser/config.rs
   ```

3. **Implement Phase 1:** Basic browser manager and scraper

4. **Test with HiveToons:** Validate approach with known Next.js site

Would you like me to start implementing Phase 1 now?
