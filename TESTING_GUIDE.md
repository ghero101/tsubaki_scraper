# Testing Guide - Source Validation

## Current Status: 26/61 Working (42.6%)

### Quick Test Commands

**Fast test (browser disabled):**
```bash
# Linux/macOS
MANGA_SCRAPER_USE_BROWSER=0 cargo test --test source_validation_test -- --ignored --nocapture

# Windows PowerShell
$env:MANGA_SCRAPER_USE_BROWSER = "0"
cargo test --test source_validation_test -- --ignored --nocapture
```

**Full test (browser enabled):**
```bash
cargo test --test source_validation_test -- --ignored --nocapture --test-threads=1
```

## Understanding Test Results

### Status Types

- **WORKING** - Source returns manga and chapters successfully
- **NO_DATA** - Source responds but returns no manga
- **ERROR** - Network/connection error
- **SSL_ERROR** - Certificate/TLS error
- **TIMEOUT** - Request took too long
- **FORBIDDEN** - 403 error (blocked)

### What the Numbers Mean

```json
{
  "source_name": "MangaDex",
  "status": "WORKING",
  "manga_count": 10,        // Number of manga found
  "total_chapters": 1582,   // Total chapters across 3 manga tested
  "duration_ms": 9631,      // Time taken in milliseconds
  "sample_manga": [...]     // First 3 manga titles found
}
```

**Important:** The test only checks the first 3 manga for chapters. So:
- `total_chapters: 3` = ~1 chapter per manga (possibly incomplete)
- `total_chapters: 150` = ~50 chapters per manga (good coverage)

## Categories of Issues

### 1. Low Chapter Counts (9 sources - 3 total chapters)

These sources find manga but only 1 chapter per series:

- StoneScape
- Asmotoon
- HiveToons
- KenScans
- QIScans
- NyxScans
- AsuraScans
- MavinTranslations
- FlameComics

**Possible causes:**
- Chapters loaded via JavaScript (need browser)
- Chapters in AJAX requests (need to find endpoint)
- Limited free content
- Wrong selectors

**How to investigate:**
1. Visit the site manually
2. Check a manga page - can you see chapters?
3. Open browser DevTools → Network tab
4. Look for AJAX requests when chapters load
5. Check if chapters appear without JavaScript (disable JS in DevTools)

### 2. Browser-Based Sources (7 sources - Long timeouts)

These timeout because browser initialization takes 30+ seconds:

- DrakeComic (28s)
- MadaraScans (28s)
- ThunderScans (54s)
- SirenScans (52s)
- VortexScans (55s)
- TempleScan (49s)
- Kagane (6s)

**Solutions:**
- Install Chrome/Chromium (see BROWSER_SETUP.md)
- Wait for auto-download (first run, ~5 minutes)
- Improve HTTP fallbacks

### 3. Commercial Sources (21 sources - Fast NO_DATA)

These respond quickly (<2s) but return no data:

- Most commercial publishers (Kodansha, Yen Press, Seven Seas, etc.)
- Paid platforms (Lezhin, Toomics, Tappytoon, etc.)
- Web platforms (Webtoon, Tapas, Webcomics)

**Likely reasons:**
- No free content available
- Paywall/login required
- Wrong selectors (check manually)

### 4. SSL/Network Errors (2 sources)

- GrimScans - TLS handshake error
- Kana - Certificate trust error

**Solutions:**
- Update certificate bundle
- Add SSL verification bypass (not recommended)
- Site may be misconfigured

## Manual Testing Individual Sources

To test a single source:

```bash
# Create a test file
cat > test_single_source.rs << 'EOF'
use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    match rust_manga_scraper::sources::asurascans::search_manga_with_urls(&client, "").await {
        Ok(manga) => {
            println!("Found {} manga", manga.len());
            for (m, url) in manga.iter().take(3) {
                println!("- {}: {}", m.title, url);

                match rust_manga_scraper::sources::asurascans::get_chapters(&client, url).await {
                    Ok(chapters) => println!("  Chapters: {}", chapters.len()),
                    Err(e) => println!("  Chapter error: {}", e),
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
EOF

# Run it
cargo run --bin test_single_source
```

## Debugging Tips

### Enable Logging

```bash
RUST_LOG=debug cargo test --test source_validation_test -- --ignored --nocapture
```

### Check Network Issues

```bash
# Test if site is accessible
curl -I "https://asuracomic.net"

# Check with user agent
curl -I -H "User-Agent: Mozilla/5.0" "https://asuracomic.net"

# Test with timeout
curl --max-time 5 "https://asuracomic.net/manga/?page=1"
```

### Common Fixes

**Problem:** "No manga returned"
- Check selectors in source file
- Visit site manually - has structure changed?
- Check if site requires JavaScript

**Problem:** "Timeout"
- Site might be slow or down
- Try increasing timeout in test
- Check internet connection

**Problem:** "SSL error"
- Site has certificate issues
- Try bypassing (temporarily): set `danger_accept_invalid_certs(true)`

**Problem:** "403 Forbidden"
- Site blocking scrapers
- Need better User-Agent
- May need browser rendering

## Performance Optimization

### Parallel Testing (Faster)

```bash
# Test in parallel (default)
cargo test --test source_validation_test -- --ignored
```

### Sequential Testing (Safer, avoids rate limits)

```bash
cargo test --test source_validation_test -- --ignored --test-threads=1
```

### Quick Smoke Test (Just a few sources)

Edit the test file to comment out most sources, or create a minimal test:

```rust
#[tokio::test]
async fn quick_test() {
    let client = Client::builder().timeout(Duration::from_secs(30)).build().unwrap();

    // Test just a few fast sources
    test_source!(mangadex, "MangaDex");
    test_source!(firescans, "FireScans");
    test_source!(anilist, "AniList");
}
```

## Expected Realistic Results

- **40-45 working sources** - Excellent (with browser setup)
- **35-40 working sources** - Good (HTTP only)
- **25-30 working sources** - Acceptable (basic setup)

Many sources legitimately can't work without:
- Browser rendering
- Authentication
- Payment/subscription
- Site cooperation (not blocking scrapers)

Reaching 100% isn't realistic for a web scraper.
