use reqwest::{Client, ClientBuilder, Response};
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;

/// User agents to rotate through to avoid bot detection
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0",
];

/// Configuration for HTTP client with bot detection bypass
#[derive(Clone)]
pub struct HttpClientConfig {
    pub timeout: Duration,
    pub max_retries: usize,
    pub initial_retry_delay_ms: u64,
    pub max_retry_delay_ms: u64,
    pub enable_cookies: bool,
    pub enable_gzip: bool,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 4,
            initial_retry_delay_ms: 500,
            max_retry_delay_ms: 8000,
            enable_cookies: true,
            enable_gzip: true,
        }
    }
}

/// Enhanced HTTP client with bot detection bypass capabilities
pub struct EnhancedHttpClient {
    client: Client,
    config: HttpClientConfig,
}

impl EnhancedHttpClient {
    /// Create a new enhanced HTTP client with default configuration
    pub fn new() -> Result<Self, reqwest::Error> {
        Self::with_config(HttpClientConfig::default())
    }

    /// Create a new enhanced HTTP client with custom configuration
    pub fn with_config(config: HttpClientConfig) -> Result<Self, reqwest::Error> {
        let mut builder = ClientBuilder::new()
            .timeout(config.timeout)
            .user_agent(Self::random_user_agent())
            .cookie_store(config.enable_cookies)
            .gzip(config.enable_gzip)
            .brotli(true)
            // Mimic browser TLS behavior
            .http2_prior_knowledge()
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .pool_idle_timeout(Some(Duration::from_secs(90)));

        // Add default headers that mimic a real browser
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8".parse().unwrap());
        headers.insert("Accept-Language", "en-US,en;q=0.9".parse().unwrap());
        headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
        headers.insert("DNT", "1".parse().unwrap());
        headers.insert("Connection", "keep-alive".parse().unwrap());
        headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());
        headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
        headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
        headers.insert("Sec-Fetch-Site", "none".parse().unwrap());
        headers.insert("Sec-Fetch-User", "?1".parse().unwrap());
        headers.insert("Cache-Control", "max-age=0".parse().unwrap());

        builder = builder.default_headers(headers);

        let client = builder.build()?;

        Ok(Self { client, config })
    }

    /// Get a random user agent from the pool
    fn random_user_agent() -> &'static str {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..USER_AGENTS.len());
        USER_AGENTS[index]
    }

    /// Calculate retry delay with exponential backoff and jitter
    fn calculate_retry_delay(&self, attempt: usize) -> Duration {
        let base_delay = self.config.initial_retry_delay_ms;
        let max_delay = self.config.max_retry_delay_ms;

        // Exponential backoff: base_delay * 2^attempt
        let delay_ms = (base_delay * 2u64.pow(attempt as u32)).min(max_delay);

        // Add jitter (±25% randomness) to avoid thundering herd
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(0.75..=1.25);
        let final_delay_ms = (delay_ms as f64 * jitter) as u64;

        Duration::from_millis(final_delay_ms)
    }

    /// Check if a status code is retryable
    fn is_retryable_status(status: reqwest::StatusCode) -> bool {
        matches!(
            status.as_u16(),
            // Rate limiting
            429 |
            // Server errors
            500 | 502 | 503 | 504 |
            // Cloudflare errors
            520 | 521 | 522 | 523 | 524 | 525 | 526 | 527
        )
    }

    /// Fetch a URL with retry logic and bot detection bypass
    pub async fn get_with_retry(&self, url: &str) -> Result<Response, reqwest::Error> {
        self.get_with_retry_and_headers(url, None).await
    }

    /// Fetch a URL with custom headers and retry logic
    pub async fn get_with_retry_and_headers(
        &self,
        url: &str,
        extra_headers: Option<reqwest::header::HeaderMap>,
    ) -> Result<Response, reqwest::Error> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            // Rotate user agent for each retry
            let mut request = self.client
                .get(url)
                .header("User-Agent", Self::random_user_agent());

            // Add extra headers if provided
            if let Some(ref headers) = extra_headers {
                request = request.headers(headers.clone());
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();

                    // If we get a retryable error status, retry
                    if Self::is_retryable_status(status) && attempt < self.config.max_retries {
                        log::warn!(
                            "Received retryable status {} for {}, attempt {}/{}",
                            status,
                            url,
                            attempt + 1,
                            self.config.max_retries + 1
                        );

                        let delay = self.calculate_retry_delay(attempt);
                        sleep(delay).await;
                        continue;
                    }

                    // Return the response (even if it's an error status that's not retryable)
                    return Ok(response);
                }
                Err(e) => {
                    // Check if it's a network error that's worth retrying
                    let should_retry = e.is_timeout()
                        || e.is_connect()
                        || e.is_request()
                        || e.status().map(Self::is_retryable_status).unwrap_or(false);

                    if should_retry && attempt < self.config.max_retries {
                        log::warn!(
                            "Request failed for {}, attempt {}/{}: {}",
                            url,
                            attempt + 1,
                            self.config.max_retries + 1,
                            e
                        );

                        let delay = self.calculate_retry_delay(attempt);
                        sleep(delay).await;
                        last_error = Some(e);
                        continue;
                    }

                    // Not retryable or out of retries
                    return Err(e);
                }
            }
        }

        // All retries exhausted
        Err(last_error.unwrap())
    }

    /// Fetch a URL and return the response text
    pub async fn get_text(&self, url: &str) -> Result<String, reqwest::Error> {
        let response = self.get_with_retry(url).await?;
        response.text().await
    }

    /// Fetch a URL with custom headers and return the response text
    pub async fn get_text_with_headers(
        &self,
        url: &str,
        headers: reqwest::header::HeaderMap,
    ) -> Result<String, reqwest::Error> {
        let response = self.get_with_retry_and_headers(url, Some(headers)).await?;
        response.text().await
    }

    /// Get the underlying reqwest client for direct access
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Add rate limiting delay between requests (call this after each request)
    pub async fn rate_limit_delay(&self, delay_ms: u64) {
        sleep(Duration::from_millis(delay_ms)).await;
    }
}

impl Default for EnhancedHttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = EnhancedHttpClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_random_user_agent() {
        let ua1 = EnhancedHttpClient::random_user_agent();
        let ua2 = EnhancedHttpClient::random_user_agent();
        assert!(USER_AGENTS.contains(&ua1));
        assert!(USER_AGENTS.contains(&ua2));
    }

    #[tokio::test]
    async fn test_retry_delay_calculation() {
        let config = HttpClientConfig::default();
        let client = EnhancedHttpClient::with_config(config).unwrap();

        let delay0 = client.calculate_retry_delay(0);
        let delay1 = client.calculate_retry_delay(1);
        let delay2 = client.calculate_retry_delay(2);

        // Each delay should be roughly double the previous (with jitter)
        assert!(delay0.as_millis() > 0);
        assert!(delay1.as_millis() >= delay0.as_millis());
        assert!(delay2.as_millis() >= delay1.as_millis());
    }

    #[test]
    fn test_retryable_status() {
        assert!(EnhancedHttpClient::is_retryable_status(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert!(EnhancedHttpClient::is_retryable_status(reqwest::StatusCode::INTERNAL_SERVER_ERROR));
        assert!(EnhancedHttpClient::is_retryable_status(reqwest::StatusCode::BAD_GATEWAY));
        assert!(EnhancedHttpClient::is_retryable_status(reqwest::StatusCode::SERVICE_UNAVAILABLE));
        assert!(EnhancedHttpClient::is_retryable_status(reqwest::StatusCode::GATEWAY_TIMEOUT));
        assert!(!EnhancedHttpClient::is_retryable_status(reqwest::StatusCode::NOT_FOUND));
        assert!(!EnhancedHttpClient::is_retryable_status(reqwest::StatusCode::FORBIDDEN));
    }
}
