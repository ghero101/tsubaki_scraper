use std::time::Duration;

/// Configuration for browser instances
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    /// Run browser in headless mode
    pub headless: bool,

    /// Browser window size
    pub window_size: (u32, u32),

    /// Custom user agent
    pub user_agent: Option<String>,

    /// Navigation timeout in seconds
    pub timeout_seconds: u64,

    /// Disable image loading for performance
    pub disable_images: bool,

    /// Additional Chrome flags
    pub chrome_flags: Vec<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            window_size: (1920, 1080),
            user_agent: Some(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"
                    .to_string(),
            ),
            timeout_seconds: 30,
            disable_images: true,
            chrome_flags: vec![],
        }
    }
}

impl BrowserConfig {
    /// Create a configuration optimized for stealth mode
    /// Useful for bypassing anti-bot protection
    pub fn stealth_mode() -> Self {
        let mut config = Self::default();
        config.chrome_flags = vec![
            "--disable-blink-features=AutomationControlled".to_string(),
            "--disable-dev-shm-usage".to_string(),
            "--no-sandbox".to_string(),
        ];
        config
    }

    /// Create a configuration for debugging (non-headless, visible browser)
    pub fn debug_mode() -> Self {
        let mut config = Self::default();
        config.headless = false;
        config.disable_images = false;
        config
    }

    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BrowserConfig::default();
        assert!(config.headless);
        assert_eq!(config.window_size, (1920, 1080));
        assert!(config.user_agent.is_some());
    }

    #[test]
    fn test_stealth_mode() {
        let config = BrowserConfig::stealth_mode();
        assert!(!config.chrome_flags.is_empty());
        assert!(config
            .chrome_flags
            .iter()
            .any(|f| f.contains("AutomationControlled")));
    }

    #[test]
    fn test_debug_mode() {
        let config = BrowserConfig::debug_mode();
        assert!(!config.headless);
        assert!(!config.disable_images);
    }
}
