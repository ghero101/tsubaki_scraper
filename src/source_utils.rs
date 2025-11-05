use crate::http_client::EnhancedHttpClient;
use crate::browser_client::BrowserClient;
use reqwest::Client;

/// Source fetch strategy
#[derive(Clone, Debug)]
pub enum FetchStrategy {
    /// Use standard HTTP client (fast, but may be blocked)
    Standard,
    /// Use enhanced HTTP client with retry and better headers
    Enhanced,
    /// Use headless browser (slow, but bypasses JS and Cloudflare)
    Browser,
}

/// Utility functions for sources to fetch content with bot detection bypass
pub struct SourceFetcher {
    enhanced_client: EnhancedHttpClient,
}

impl SourceFetcher {
    /// Create a new source fetcher
    pub fn new() -> Result<Self, reqwest::Error> {
        let enhanced_client = EnhancedHttpClient::new()?;
        Ok(Self { enhanced_client })
    }

    /// Fetch HTML content using the specified strategy
    pub async fn fetch_html(
        &self,
        url: &str,
        strategy: FetchStrategy,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match strategy {
            FetchStrategy::Standard => {
                // Legacy behavior for backward compatibility
                let client = Client::new();
                Ok(client.get(url).send().await?.text().await?)
            }
            FetchStrategy::Enhanced => {
                // Use enhanced HTTP client with retry and better headers
                Ok(self.enhanced_client.get_text(url).await?)
            }
            FetchStrategy::Browser => {
                // Use headless browser for JavaScript-rendered content
                let browser = BrowserClient::new()?;
                browser.get_html(url)
            }
        }
    }

    /// Fetch HTML with automatic strategy detection based on URL/domain
    pub async fn fetch_html_auto(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let strategy = self.detect_strategy(url);
        self.fetch_html(url, strategy).await
    }

    /// Detect the best fetch strategy for a given URL
    fn detect_strategy(&self, url: &str) -> FetchStrategy {
        // Domains known to require headless browser
        let browser_required = [
            "asmotoon.com",
            "hivetoons.com",
            "kenscans.com",
            "qiscans.org",
            "nyxscans.com",
        ];

        // Domains known to have Cloudflare protection
        let cloudflare_protected = [
            "drakecomic.com",
            "madarascans.com",
            "rizzfables.com",
        ];

        for domain in &browser_required {
            if url.contains(domain) {
                return FetchStrategy::Browser;
            }
        }

        for domain in &cloudflare_protected {
            if url.contains(domain) {
                // Try enhanced first, fall back to browser if needed
                return FetchStrategy::Enhanced;
            }
        }

        // Default to enhanced for better reliability
        FetchStrategy::Enhanced
    }

    /// Fetch with Cloudflare bypass using browser
    pub async fn fetch_with_cloudflare_bypass(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let browser = BrowserClient::new()?;
        browser.navigate_with_cloudflare_bypass(url)
    }

    /// Get the underlying enhanced HTTP client
    pub fn http_client(&self) -> &EnhancedHttpClient {
        &self.enhanced_client
    }

    /// Get a standard reqwest client (for backward compatibility)
    pub fn standard_client(&self) -> &Client {
        self.enhanced_client.client()
    }
}

impl Default for SourceFetcher {
    fn default() -> Self {
        Self::new().expect("Failed to create source fetcher")
    }
}

/// Helper function to create an enhanced HTTP client for sources
pub fn create_enhanced_client() -> Result<EnhancedHttpClient, reqwest::Error> {
    EnhancedHttpClient::new()
}

/// Helper function to fetch HTML with retry logic (for easy migration)
pub async fn fetch_html_with_retry(
    client: &EnhancedHttpClient,
    url: &str,
) -> Result<String, reqwest::Error> {
    client.get_text(url).await
}

/// Helper function to fetch HTML using browser (for easy migration)
pub async fn fetch_html_with_browser(
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let browser = BrowserClient::new()?;
    browser.get_html(url)
}

/// Helper function to fetch HTML with Cloudflare bypass
pub async fn fetch_html_cloudflare_bypass(
    url: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let browser = BrowserClient::new()?;
    browser.navigate_with_cloudflare_bypass(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_detection() {
        let fetcher = SourceFetcher::new().unwrap();

        // Browser required domains
        assert!(matches!(
            fetcher.detect_strategy("https://asmotoon.com/manga/test"),
            FetchStrategy::Browser
        ));
        assert!(matches!(
            fetcher.detect_strategy("https://hivetoons.com/series/test"),
            FetchStrategy::Browser
        ));

        // Cloudflare protected domains
        assert!(matches!(
            fetcher.detect_strategy("https://drakecomic.com/manga/test"),
            FetchStrategy::Enhanced
        ));

        // Unknown domains should use enhanced
        assert!(matches!(
            fetcher.detect_strategy("https://example.com/manga/test"),
            FetchStrategy::Enhanced
        ));
    }

    #[tokio::test]
    async fn test_fetcher_creation() {
        let fetcher = SourceFetcher::new();
        assert!(fetcher.is_ok());
    }
}
