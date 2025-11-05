use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub download_dir: String,
    #[serde(default)]
    pub bot_detection: BotDetectionConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BotDetectionConfig {
    /// Enable enhanced HTTP client with retry logic
    #[serde(default = "default_true")]
    pub enable_enhanced_client: bool,

    /// Enable headless browser for JavaScript-rendered sites
    #[serde(default = "default_false")]
    pub enable_browser: bool,

    /// Maximum number of retry attempts for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,

    /// Initial retry delay in milliseconds
    #[serde(default = "default_initial_retry_delay")]
    pub initial_retry_delay_ms: u64,

    /// Maximum retry delay in milliseconds
    #[serde(default = "default_max_retry_delay")]
    pub max_retry_delay_ms: u64,

    /// Timeout for HTTP requests in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Enable cookie support
    #[serde(default = "default_true")]
    pub enable_cookies: bool,

    /// Enable gzip/brotli compression
    #[serde(default = "default_true")]
    pub enable_compression: bool,

    /// Browser timeout in seconds (for JavaScript-rendered sites)
    #[serde(default = "default_browser_timeout")]
    pub browser_timeout_secs: u64,

    /// Browser headless mode
    #[serde(default = "default_true")]
    pub browser_headless: bool,

    /// Disable images in browser (faster loading)
    #[serde(default = "default_true")]
    pub browser_disable_images: bool,

    /// Rate limiting delay between requests in milliseconds
    #[serde(default = "default_rate_limit")]
    pub rate_limit_delay_ms: u64,
}

fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_max_retries() -> usize { 4 }
fn default_initial_retry_delay() -> u64 { 500 }
fn default_max_retry_delay() -> u64 { 8000 }
fn default_timeout() -> u64 { 30 }
fn default_browser_timeout() -> u64 { 30 }
fn default_rate_limit() -> u64 { 300 }

impl Default for BotDetectionConfig {
    fn default() -> Self {
        Self {
            enable_enhanced_client: true,
            enable_browser: false, // Disabled by default (requires Chrome)
            max_retries: 4,
            initial_retry_delay_ms: 500,
            max_retry_delay_ms: 8000,
            timeout_secs: 30,
            enable_cookies: true,
            enable_compression: true,
            browser_timeout_secs: 30,
            browser_headless: true,
            browser_disable_images: true,
            rate_limit_delay_ms: 300,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_dir: "downloads".to_string(),
            bot_detection: BotDetectionConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Path::new("config.toml");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(cfg) = toml::from_str::<Config>(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }
}

impl BotDetectionConfig {
    /// Create an enhanced HTTP client from this configuration
    pub fn create_http_client(&self) -> Result<crate::http_client::EnhancedHttpClient, reqwest::Error> {
        use crate::http_client::{EnhancedHttpClient, HttpClientConfig};
        use std::time::Duration;

        let config = HttpClientConfig {
            timeout: Duration::from_secs(self.timeout_secs),
            max_retries: self.max_retries,
            initial_retry_delay_ms: self.initial_retry_delay_ms,
            max_retry_delay_ms: self.max_retry_delay_ms,
            enable_cookies: self.enable_cookies,
            enable_gzip: self.enable_compression,
        };

        EnhancedHttpClient::with_config(config)
    }

    /// Create a browser client from this configuration
    pub fn create_browser_client(&self) -> Result<crate::browser_client::BrowserClient, Box<dyn std::error::Error>> {
        use crate::browser_client::{BrowserClient, BrowserConfig};
        use std::time::Duration;

        if !self.enable_browser {
            return Err("Browser client is disabled in configuration".into());
        }

        let config = BrowserConfig {
            headless: self.browser_headless,
            window_width: 1920,
            window_height: 1080,
            timeout: Duration::from_secs(self.browser_timeout_secs),
            disable_images: self.browser_disable_images,
            user_agent: Some(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
                    .to_string(),
            ),
        };

        BrowserClient::with_config(config)
    }
}
