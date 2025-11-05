use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::Arc;
use std::time::Duration;

/// Configuration for headless browser
#[derive(Clone)]
pub struct BrowserConfig {
    pub headless: bool,
    pub window_width: u32,
    pub window_height: u32,
    pub timeout: Duration,
    pub disable_images: bool,
    pub user_agent: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            window_width: 1920,
            window_height: 1080,
            timeout: Duration::from_secs(30),
            disable_images: true, // Faster loading
            user_agent: Some(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
                    .to_string(),
            ),
        }
    }
}

/// Enhanced browser client for JavaScript-rendered sites
pub struct BrowserClient {
    browser: Browser,
    config: BrowserConfig,
}

impl BrowserClient {
    /// Create a new browser client with default configuration
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config(BrowserConfig::default())
    }

    /// Create a new browser client with custom configuration
    pub fn with_config(config: BrowserConfig) -> Result<Self, Box<dyn std::error::Error>> {
        use std::ffi::OsStr;

        // Store all owned strings first for lifetime management
        let images_arg = if config.disable_images {
            Some("--blink-settings=imagesEnabled=false".to_string())
        } else {
            None
        };

        let user_agent_arg = config.user_agent.as_ref().map(|ua| format!("--user-agent={}", ua));

        // Build list of chrome arguments
        let mut args: Vec<&OsStr> = vec![
            OsStr::new("--disable-blink-features=AutomationControlled"),
            OsStr::new("--disable-dev-shm-usage"),
            OsStr::new("--no-sandbox"),
            OsStr::new("--disable-setuid-sandbox"),
            OsStr::new("--disable-web-security"),
            OsStr::new("--disable-features=IsolateOrigins,site-per-process"),
        ];

        if let Some(ref img) = images_arg {
            args.push(OsStr::new(img));
        }

        if let Some(ref ua) = user_agent_arg {
            args.push(OsStr::new(ua));
        }

        let launch_options = LaunchOptions::default_builder()
            .headless(config.headless)
            .window_size(Some((config.window_width, config.window_height)))
            .args(args)
            .build()?;

        let browser = Browser::new(launch_options)?;

        Ok(Self { browser, config })
    }

    /// Create a new tab/page
    fn create_tab(&self) -> Result<Arc<Tab>, Box<dyn std::error::Error>> {
        let tab = self.browser.new_tab()?;

        // Set viewport
        tab.set_bounds(headless_chrome::types::Bounds::Normal {
            left: Some(0),
            top: Some(0),
            width: Some(self.config.window_width as f64),
            height: Some(self.config.window_height as f64),
        })?;

        // Override navigator properties to avoid detection
        let stealth_script = r#"
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined
            });
            Object.defineProperty(navigator, 'plugins', {
                get: () => [1, 2, 3, 4, 5]
            });
            Object.defineProperty(navigator, 'languages', {
                get: () => ['en-US', 'en']
            });
        "#;

        tab.evaluate(stealth_script, false)?;

        Ok(tab)
    }

    /// Navigate to a URL and wait for the page to load
    pub fn navigate(&self, url: &str) -> Result<Arc<Tab>, Box<dyn std::error::Error>> {
        log::info!("Browser navigating to: {}", url);

        let tab = self.create_tab()?;

        tab.navigate_to(url)?
            .wait_until_navigated()?;

        // Wait for network idle (no requests for 500ms)
        tab.wait_for_element_with_custom_timeout("body", self.config.timeout)?;

        Ok(tab)
    }

    /// Navigate to a URL and return the page HTML
    pub fn get_html(&self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let tab = self.navigate(url)?;

        // Wait a bit for JavaScript to execute
        std::thread::sleep(Duration::from_millis(1000));

        let html = tab.get_content()?;
        Ok(html)
    }

    /// Navigate to a URL, wait for a selector, and return the page HTML
    pub fn get_html_wait_for(
        &self,
        url: &str,
        selector: &str,
        timeout: Option<Duration>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tab = self.navigate(url)?;

        let wait_timeout = timeout.unwrap_or(self.config.timeout);

        // Wait for the specific element
        tab.wait_for_element_with_custom_timeout(selector, wait_timeout)?;

        // Additional wait for dynamic content
        std::thread::sleep(Duration::from_millis(500));

        let html = tab.get_content()?;
        Ok(html)
    }

    /// Execute JavaScript and return the result
    pub fn execute_script(
        &self,
        url: &str,
        script: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tab = self.navigate(url)?;

        let result = tab.evaluate(script, true)?;
        Ok(serde_json::to_string(&result.value)?)
    }

    /// Check if Cloudflare challenge is present
    pub fn has_cloudflare_challenge(&self, tab: &Arc<Tab>) -> bool {
        // Check for common Cloudflare challenge indicators
        let indicators = vec![
            "#cf-challenge-running",
            ".cf-browser-verification",
            "#challenge-running",
            ".challenge-form",
        ];

        for selector in indicators {
            if tab.wait_for_element_with_custom_timeout(selector, Duration::from_secs(2)).is_ok() {
                return true;
            }
        }

        false
    }

    /// Navigate and automatically handle Cloudflare challenges
    pub fn navigate_with_cloudflare_bypass(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        log::info!("Navigating with Cloudflare bypass to: {}", url);

        let tab = self.navigate(url)?;

        // Check for Cloudflare challenge
        if self.has_cloudflare_challenge(&tab) {
            log::info!("Cloudflare challenge detected, waiting for bypass...");

            // Wait up to 30 seconds for the challenge to complete
            let max_wait = Duration::from_secs(30);
            let start = std::time::Instant::now();

            while start.elapsed() < max_wait {
                std::thread::sleep(Duration::from_secs(1));

                // Check if challenge is still present
                if !self.has_cloudflare_challenge(&tab) {
                    log::info!("Cloudflare challenge bypassed!");
                    break;
                }

                if start.elapsed() >= max_wait {
                    return Err("Cloudflare challenge timeout".into());
                }
            }

            // Wait a bit more for the page to fully load after bypass
            std::thread::sleep(Duration::from_millis(2000));
        }

        let html = tab.get_content()?;
        Ok(html)
    }

    /// Take a screenshot of the page (useful for debugging)
    pub fn screenshot(
        &self,
        url: &str,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tab = self.navigate(url)?;

        let screenshot_data = tab.capture_screenshot(
            headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true,
        )?;

        std::fs::write(output_path, screenshot_data)?;
        log::info!("Screenshot saved to: {}", output_path);

        Ok(())
    }

    /// Close the browser
    pub fn close(self) -> Result<(), Box<dyn std::error::Error>> {
        // Browser will be dropped and closed automatically
        Ok(())
    }
}

impl Drop for BrowserClient {
    fn drop(&mut self) {
        // Browser cleanup happens automatically
        log::debug!("Browser client dropped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_config_default() {
        let config = BrowserConfig::default();
        assert_eq!(config.headless, true);
        assert_eq!(config.window_width, 1920);
        assert_eq!(config.window_height, 1080);
        assert_eq!(config.disable_images, true);
    }

    #[test]
    #[ignore] // Ignore by default as it requires Chrome/Chromium
    fn test_browser_creation() {
        let client = BrowserClient::new();
        assert!(client.is_ok());
    }

    #[test]
    #[ignore] // Ignore by default as it requires Chrome/Chromium and internet
    fn test_simple_navigation() {
        let client = BrowserClient::new().unwrap();
        let result = client.get_html("https://example.com");
        assert!(result.is_ok());
        let html = result.unwrap();
        assert!(html.contains("Example Domain"));
    }
}
