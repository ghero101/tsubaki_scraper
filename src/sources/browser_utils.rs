/// Helper utilities for browser-based sources

/// Check if browser should be used based on environment variable
/// Returns false if MANGA_SCRAPER_USE_BROWSER is set to "0" or "false"
///
/// Example usage:
/// ```sh
/// export MANGA_SCRAPER_USE_BROWSER=0  # Disable browser rendering globally
/// cargo test
/// ```
#[allow(dead_code)]
pub fn should_use_browser() -> bool {
    std::env::var("MANGA_SCRAPER_USE_BROWSER")
        .map(|v| v != "0" && v.to_lowercase() != "false")
        .unwrap_or(true)
}
