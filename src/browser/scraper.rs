use super::manager::BrowserError;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::Tab;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// High-level scraping utilities for a browser tab
pub struct BrowserScraper {
    tab: Arc<Tab>,
    default_timeout: Duration,
}

impl BrowserScraper {
    /// Create a new scraper with the given tab
    pub fn new(tab: Arc<Tab>) -> Self {
        Self {
            tab,
            default_timeout: Duration::from_secs(30),
        }
    }

    /// Create a new scraper with a custom default timeout
    pub fn with_timeout(tab: Arc<Tab>, timeout: Duration) -> Self {
        Self {
            tab,
            default_timeout: timeout,
        }
    }

    /// Navigate to a URL and wait for page load
    pub fn navigate(&self, url: &str) -> Result<(), BrowserError> {
        self.tab
            .navigate_to(url)
            .map_err(|e| BrowserError::NavigationError(format!("Failed to navigate to {}: {}", url, e)))?;

        self.tab
            .wait_until_navigated()
            .map_err(|e| BrowserError::NavigationError(format!("Navigation timeout for {}: {}", url, e)))?;

        Ok(())
    }

    /// Wait for an element matching the given CSS selector
    pub fn wait_for_selector(&self, selector: &str) -> Result<(), BrowserError> {
        self.wait_for_selector_with_timeout(selector, self.default_timeout)
    }

    /// Wait for an element with a custom timeout
    pub fn wait_for_selector_with_timeout(
        &self,
        selector: &str,
        timeout: Duration,
    ) -> Result<(), BrowserError> {
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(BrowserError::Timeout(format!(
                    "Waiting for selector: {}",
                    selector
                )));
            }

            // Try to find the element
            let script = format!(
                r#"document.querySelector('{}') !== null"#,
                selector.replace('\'', "\\'")
            );

            match self.tab.evaluate(&script, false) {
                Ok(result) => {
                    if let Some(value) = result.value {
                        if value.as_bool() == Some(true) {
                            return Ok(());
                        }
                    }
                }
                Err(_) => {
                    // Element not found yet, continue waiting
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Get the HTML content of the page
    pub fn get_html(&self) -> Result<String, BrowserError> {
        self.tab
            .get_content()
            .map_err(|e| BrowserError::HtmlExtractionError(e.to_string()))
    }

    /// Execute JavaScript and return the result as a string
    pub fn evaluate_script(&self, script: &str) -> Result<String, BrowserError> {
        let result = self
            .tab
            .evaluate(script, false)
            .map_err(|e| BrowserError::JavaScriptError(e.to_string()))?;

        result
            .value
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .ok_or_else(|| BrowserError::JavaScriptError("Script returned no value".to_string()))
    }

    /// Click an element matching the given selector
    pub fn click(&self, selector: &str) -> Result<(), BrowserError> {
        let script = format!(
            r#"document.querySelector('{}').click();"#,
            selector.replace('\'', "\\'")
        );

        self.tab
            .evaluate(&script, false)
            .map_err(|e| BrowserError::JavaScriptError(format!("Click failed: {}", e)))?;

        Ok(())
    }

    /// Scroll to the bottom of the page
    /// Useful for lazy-loaded content
    pub fn scroll_to_bottom(&self) -> Result<(), BrowserError> {
        let script = "window.scrollTo(0, document.body.scrollHeight);";

        self.tab
            .evaluate(script, false)
            .map_err(|e| BrowserError::JavaScriptError(format!("Scroll failed: {}", e)))?;

        // Wait for potential lazy loading
        std::thread::sleep(Duration::from_millis(500));

        Ok(())
    }

    /// Wait for Cloudflare challenge to complete
    /// Returns when the page title no longer contains "Just a moment"
    pub fn wait_for_cloudflare(&self, timeout: Duration) -> Result<(), BrowserError> {
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(BrowserError::Timeout(
                    "Cloudflare challenge timeout".to_string(),
                ));
            }

            if let Ok(title) = self.tab.get_title() {
                if !title.to_lowercase().contains("just a moment") {
                    // Additional wait to ensure page is fully loaded
                    std::thread::sleep(Duration::from_secs(2));
                    return Ok(());
                }
            }

            std::thread::sleep(Duration::from_millis(500));
        }
    }

    /// Take a screenshot of the current page (useful for debugging)
    pub fn screenshot(&self, path: &str) -> Result<(), BrowserError> {
        let screenshot_data = self
            .tab
            .capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)
            .map_err(|e| BrowserError::JavaScriptError(format!("Screenshot failed: {}", e)))?;

        std::fs::write(path, screenshot_data)
            .map_err(|e| BrowserError::JavaScriptError(format!("Failed to save screenshot: {}", e)))?;

        Ok(())
    }

    /// Get a reference to the underlying tab
    pub fn tab(&self) -> &Arc<Tab> {
        &self.tab
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::{BrowserConfig, BrowserManager};

    #[test]
    #[ignore] // Requires Chrome to be installed
    fn test_basic_navigation() {
        let manager = BrowserManager::new(BrowserConfig::default()).unwrap();
        let tab = manager.new_tab().unwrap();
        let scraper = BrowserScraper::new(tab);

        assert!(scraper.navigate("https://example.com").is_ok());
    }

    #[test]
    #[ignore] // Requires Chrome to be installed
    fn test_html_extraction() {
        let manager = BrowserManager::new(BrowserConfig::default()).unwrap();
        let tab = manager.new_tab().unwrap();
        let scraper = BrowserScraper::new(tab);

        scraper.navigate("https://example.com").unwrap();
        let html = scraper.get_html().unwrap();

        assert!(html.contains("Example"));
        assert!(html.len() > 100);
    }

    #[test]
    #[ignore] // Requires Chrome to be installed
    fn test_wait_for_selector() {
        let manager = BrowserManager::new(BrowserConfig::default()).unwrap();
        let tab = manager.new_tab().unwrap();
        let scraper = BrowserScraper::new(tab);

        scraper.navigate("https://example.com").unwrap();
        assert!(scraper.wait_for_selector("h1").is_ok());
    }
}
