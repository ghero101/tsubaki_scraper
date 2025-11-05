# Bot Detection Bypass and Headless Browser Integration

This document describes the bot detection bypass features and headless browser integration added to the manga scraper.

## Overview

The scraper now includes three main components to bypass bot detection and handle JavaScript-rendered content:

1. **Enhanced HTTP Client** (`src/http_client.rs`) - Improved HTTP client with retry logic and realistic browser headers
2. **Browser Client** (`src/browser_client.rs`) - Headless Chrome integration for JavaScript-rendered sites
3. **Source Utilities** (`src/source_utils.rs`) - Helper functions to easily integrate these features

## Features

### Enhanced HTTP Client

The `EnhancedHttpClient` provides:

- **Automatic retry with exponential backoff** - Retries failed requests with increasing delays
- **Realistic browser headers** - Mimics real browser requests to avoid detection
- **Rotating User-Agents** - Uses different user agents for each request
- **Cookie jar support** - Maintains cookies across requests
- **Cloudflare error handling** - Automatically retries on Cloudflare errors (520-527)
- **Rate limiting detection** - Handles 429 (Too Many Requests) responses

#### Usage Example

```rust
use crate::http_client::EnhancedHttpClient;

// Create a client
let client = EnhancedHttpClient::new()?;

// Fetch HTML with automatic retry
let html = client.get_text("https://example.com/manga").await?;

// Fetch with custom headers
let mut headers = reqwest::header::HeaderMap::new();
headers.insert("Referer", "https://example.com".parse()?);
let html = client.get_text_with_headers("https://example.com/manga/123", headers).await?;

// Add rate limiting between requests
client.rate_limit_delay(300).await;
```

### Browser Client

The `BrowserClient` provides headless Chrome automation for sites that require JavaScript:

- **JavaScript execution** - Renders JavaScript-heavy pages
- **Cloudflare challenge bypass** - Automatically waits for challenges to complete
- **Stealth mode** - Hides automation indicators
- **Anti-detection measures** - Overrides navigator properties
- **Screenshot capability** - Useful for debugging

#### Usage Example

```rust
use crate::browser_client::BrowserClient;

// Create a browser client
let browser = BrowserClient::new()?;

// Fetch HTML from a JavaScript-rendered page
let html = browser.get_html("https://example.com/manga").await?;

// Wait for a specific element before getting HTML
let html = browser.get_html_wait_for(
    "https://example.com/manga",
    "div.manga-list",
    Some(Duration::from_secs(10))
).await?;

// Navigate with automatic Cloudflare bypass
let html = browser.navigate_with_cloudflare_bypass("https://example.com").await?;

// Take a screenshot for debugging
browser.screenshot("https://example.com", "/tmp/debug.png")?;
```

### Source Utilities

The `SourceFetcher` provides a unified interface with automatic strategy detection:

```rust
use crate::source_utils::{SourceFetcher, FetchStrategy};

let fetcher = SourceFetcher::new()?;

// Fetch with automatic strategy detection based on domain
let html = fetcher.fetch_html_auto("https://asmotoon.com/manga/test").await?;

// Or specify a strategy explicitly
let html = fetcher.fetch_html("https://example.com", FetchStrategy::Enhanced).await?;
let html = fetcher.fetch_html("https://example.com", FetchStrategy::Browser).await?;
```

## Integration with Sources

### Updating Existing Sources

The `wp_manga.rs` module has been updated to include:
- Better browser headers
- Enhanced retry logic with longer delays
- Support for Cloudflare errors (520-524)
- Network error detection and retry

All sources using `wp_manga::search_manga_with_urls_base()` automatically benefit from these improvements.

### Adding Browser Support to a Source

For sources that require JavaScript rendering (identified: asmotoon, hivetoons, kenscans, qiscans, nyxscans):

```rust
// Standard implementation (HTTP only)
pub async fn search_manga_with_urls(client: &Client, title: &str) -> Result<Vec<(Manga, String)>, reqwest::Error> {
    let url = format!("{}/?s={}", BASE_URL, title);
    let response = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 ...")
        .send().await?.text().await?;
    Ok(parse_search_page(&response))
}

// Alternative browser implementation
pub async fn search_manga_with_urls_browser(title: &str) -> Result<Vec<(Manga, String)>, Box<dyn std::error::Error>> {
    use crate::browser_client::BrowserClient;

    let browser = BrowserClient::new()?;
    let url = format!("{}/?s={}", BASE_URL, title);
    let html = browser.get_html(&url)?;
    Ok(parse_search_page(&html))
}
```

## Known Problematic Sources

Based on testing, these sources have specific issues:

### Require JavaScript Rendering
- **asmotoon.com** - Content loaded client-side
- **hivetoons.com** - Dynamic content loading
- **kenscans.com** - Next.js app with client-side rendering
- **qiscans.org** - JavaScript-rendered manga list
- **nyxscans.com** - AJAX-based content loading

**Solution**: Use `BrowserClient` for these sources

### Cloudflare Protected
- **drakecomic.com** - Cloudflare challenge
- **madarascans.com** - Bot detection
- **rizzfables.com** - IP-based blocking

**Solution**: Use `BrowserClient` with `navigate_with_cloudflare_bypass()` or enhanced HTTP client with better headers

### DNS/SSL Issues
- **thunderscans** - Certificate errors
- **asurascans** - Domain issues
- **sirenscans** - SSL verification
- **vortexscans** - Connection timeout
- **grimscans** - DNS resolution

**Solution**: Verify domains still exist, update BASE_URL if moved

## Configuration

### HTTP Client Configuration

```rust
use crate::http_client::{EnhancedHttpClient, HttpClientConfig};
use std::time::Duration;

let config = HttpClientConfig {
    timeout: Duration::from_secs(30),
    max_retries: 4,
    initial_retry_delay_ms: 500,
    max_retry_delay_ms: 8000,
    enable_cookies: true,
    enable_gzip: true,
};

let client = EnhancedHttpClient::with_config(config)?;
```

### Browser Client Configuration

```rust
use crate::browser_client::{BrowserClient, BrowserConfig};
use std::time::Duration;

let config = BrowserConfig {
    headless: true,
    window_width: 1920,
    window_height: 1080,
    timeout: Duration::from_secs(30),
    disable_images: true,  // Faster loading
    user_agent: Some("Mozilla/5.0 ...".to_string()),
};

let browser = BrowserClient::with_config(config)?;
```

## Testing

Run the tests:

```bash
# Test HTTP client
cargo test --lib http_client

# Test browser client (requires Chrome/Chromium)
cargo test --lib browser_client -- --ignored

# Test source utilities
cargo test --lib source_utils
```

## Performance Considerations

### HTTP Client
- **Fast** - Direct HTTP requests
- **Low resource usage** - No browser overhead
- **Recommended for** - Static HTML sites, API endpoints

### Browser Client
- **Slow** - Full browser automation (2-5 seconds per page)
- **High resource usage** - Requires Chrome/Chromium
- **Use only when** - JavaScript rendering is required, Cloudflare bypass needed

## Dependencies Added

```toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "cookies", "gzip", "brotli"] }
headless_chrome = "1.0"
rand = "0.8"
```

## Troubleshooting

### "Chrome binary not found"
Install Chrome or Chromium:
```bash
# Ubuntu/Debian
sudo apt-get install chromium-browser

# macOS
brew install chromium

# Arch Linux
sudo pacman -S chromium
```

### "Request timeout"
Increase timeout in configuration:
```rust
let mut config = HttpClientConfig::default();
config.timeout = Duration::from_secs(60);
```

### "Too many redirects"
Some sites have complex redirect chains. Use browser client:
```rust
let browser = BrowserClient::new()?;
let html = browser.navigate_with_cloudflare_bypass(url).await?;
```

### "SSL certificate verification failed"
Check if the domain is still valid and the certificate is up to date. May need to update BASE_URL.

## Future Improvements

Potential enhancements:

1. **Proxy support** - Route requests through proxies
2. **CAPTCHA solving** - Integration with CAPTCHA solving services
3. **Request fingerprinting** - More advanced TLS fingerprinting
4. **Browser pooling** - Reuse browser instances for better performance
5. **Session persistence** - Save and restore browser sessions
6. **API endpoint discovery** - Find and use hidden JSON APIs instead of scraping

## References

- [reqwest documentation](https://docs.rs/reqwest/)
- [headless_chrome documentation](https://docs.rs/headless_chrome/)
- [Cloudflare bot detection](https://developers.cloudflare.com/bots/)
