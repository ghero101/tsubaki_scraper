use super::config::BrowserConfig;
use headless_chrome::{Browser, LaunchOptions, Tab};
use std::sync::Arc;

/// Manages browser instances and tab creation
pub struct BrowserManager {
    browser: Arc<Browser>,
    config: BrowserConfig,
}

impl BrowserManager {
    /// Create a new browser manager with the given configuration
    pub fn new(config: BrowserConfig) -> Result<Self, BrowserError> {
        let launch_options = Self::build_launch_options(&config)?;

        let browser = Browser::new(launch_options)
            .map_err(|e| BrowserError::InitializationError(e.to_string()))?;

        Ok(Self {
            browser: Arc::new(browser),
            config,
        })
    }

    /// Build Chrome launch options from our config
    fn build_launch_options(config: &BrowserConfig) -> Result<LaunchOptions, BrowserError> {
        // For now, use a simple approach with default options
        // The headless_chrome API makes it difficult to add custom args with proper lifetimes
        // We'll handle user agent and other settings through CDP commands after browser starts

        let options = LaunchOptions::default_builder()
            .headless(config.headless)
            .window_size(Some((config.window_size.0, config.window_size.1)))
            .build()
            .map_err(|e| BrowserError::ConfigurationError(e.to_string()))?;

        Ok(options)
    }

    /// Create a new tab for scraping
    pub fn new_tab(&self) -> Result<Arc<Tab>, BrowserError> {
        self.browser
            .new_tab()
            .map_err(|e| BrowserError::TabCreationError(e.to_string()))
    }

    /// Get the browser configuration
    pub fn config(&self) -> &BrowserConfig {
        &self.config
    }

    /// Get a reference to the underlying browser
    pub fn browser(&self) -> &Arc<Browser> {
        &self.browser
    }
}

/// Errors that can occur during browser operations
#[derive(Debug, thiserror::Error)]
pub enum BrowserError {
    #[error("Browser initialization failed: {0}")]
    InitializationError(String),

    #[error("Browser configuration error: {0}")]
    ConfigurationError(String),

    #[error("Tab creation failed: {0}")]
    TabCreationError(String),

    #[error("Navigation error: {0}")]
    NavigationError(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Timeout waiting for: {0}")]
    Timeout(String),

    #[error("JavaScript execution error: {0}")]
    JavaScriptError(String),

    #[error("HTML extraction error: {0}")]
    HtmlExtractionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_manager_creation() {
        let config = BrowserConfig::default();
        let manager = BrowserManager::new(config);

        // Note: This test may fail if Chrome/Chromium is not installed
        // In CI/CD, you'd want to skip this or use a Docker container with Chrome
        if let Ok(manager) = manager {
            assert!(manager.new_tab().is_ok());
        }
    }

    #[test]
    fn test_launch_options_build() {
        let config = BrowserConfig::default();
        let options = BrowserManager::build_launch_options(&config);
        assert!(options.is_ok());
    }

    #[test]
    fn test_stealth_mode_options() {
        let config = BrowserConfig::stealth_mode();
        let options = BrowserManager::build_launch_options(&config).unwrap();

        // Check that stealth flags are included
        assert!(options
            .args
            .iter()
            .any(|arg| arg.to_string_lossy().contains("AutomationControlled")));
    }
}
