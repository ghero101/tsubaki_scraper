/// Browser client tests
/// These tests require Chrome/Chromium to be installed
/// Run with: cargo test --test browser_client_tests -- --ignored
use rust_manga_scraper::browser_client::{BrowserClient, BrowserConfig};
use std::time::Duration;

#[test]
#[ignore] // Requires Chrome/Chromium
fn test_browser_creation() {
    let result = BrowserClient::new().await;
    assert!(
        result.is_ok(),
        "Failed to create browser client. Is Chrome/Chromium installed?"
    );
}

#[test]
#[ignore] // Requires Chrome/Chromium
fn test_browser_with_config() {
    let config = BrowserConfig {
        headless: true,
        window_width: 1280,
        window_height: 720,
        timeout: Duration::from_secs(15),
        disable_images: true,
        user_agent: Some("Test User Agent".to_string()),
    };

    let result = BrowserClient::with_config(config);
    assert!(
        result.is_ok(),
        "Failed to create browser with custom config"
    );
}

#[test]
#[ignore] // Requires Chrome/Chromium and internet
fn test_simple_navigation() {
    let browser = BrowserClient::new()
        .await
        .expect("Chrome/Chromium not installed");

    let result = browser.get_html("https://example.com");

    assert!(result.is_ok(), "Failed to navigate to example.com");

    let html = result.unwrap();
    assert!(
        html.contains("Example Domain"),
        "Page content not as expected"
    );
    assert!(html.contains("<html"), "Should contain HTML tags");
}

#[test]
#[ignore] // Requires Chrome/Chromium and internet
fn test_wait_for_element() {
    let browser = BrowserClient::new()
        .await
        .expect("Chrome/Chromium not installed");

    let result =
        browser.get_html_wait_for("https://example.com", "h1", Some(Duration::from_secs(10)));

    assert!(result.is_ok(), "Failed to wait for element");

    let html = result.unwrap();
    assert!(
        html.contains("Example Domain"),
        "Should find expected content"
    );
}

#[test]
#[ignore] // Requires Chrome/Chromium and internet
fn test_javascript_execution() {
    let browser = BrowserClient::new()
        .await
        .expect("Chrome/Chromium not installed");

    let result = browser.execute_script("https://example.com", "document.title");

    assert!(result.is_ok(), "Failed to execute JavaScript");

    let title = result.unwrap();
    assert!(title.contains("Example"), "Should get page title");
}

#[test]
#[ignore] // Requires Chrome/Chromium and internet
fn test_cloudflare_detection() {
    let browser = BrowserClient::new()
        .await
        .expect("Chrome/Chromium not installed");

    let tab = browser
        .navigate("https://example.com")
        .expect("Failed to navigate");

    // Example.com doesn't have Cloudflare, so this should return false
    let has_cloudflare = browser.has_cloudflare_challenge(&tab);
    assert!(
        !has_cloudflare,
        "Example.com should not have Cloudflare challenge"
    );
}

#[test]
#[ignore] // Requires Chrome/Chromium and internet
fn test_browser_stealth_mode() {
    let browser = BrowserClient::new()
        .await
        .expect("Chrome/Chromium not installed");

    // Execute JavaScript to check if webdriver is hidden
    let result = browser.execute_script("https://example.com", "navigator.webdriver");

    assert!(result.is_ok(), "Failed to check webdriver property");

    let webdriver_value = result.unwrap();
    // Should be undefined or null (not true) due to stealth mode
    assert!(
        webdriver_value.contains("undefined") || webdriver_value.contains("null"),
        "Webdriver should be hidden for stealth mode"
    );
}

#[test]
#[ignore] // Requires Chrome/Chromium and internet - slow test
fn test_multiple_navigations() {
    let browser = BrowserClient::new()
        .await
        .expect("Chrome/Chromium not installed");

    // Navigate to multiple pages
    let urls = vec!["https://example.com", "https://example.org"];

    for url in urls {
        let result = browser.get_html(url);
        assert!(result.is_ok(), "Failed to navigate to {}", url);

        let html = result.unwrap();
        assert!(!html.is_empty(), "HTML should not be empty for {}", url);
    }
}

#[test]
#[ignore] // Requires Chrome/Chromium
fn test_browser_timeout() {
    let config = BrowserConfig {
        headless: true,
        window_width: 1920,
        window_height: 1080,
        timeout: Duration::from_millis(100), // Very short timeout
        disable_images: true,
        user_agent: None,
    };

    let browser = BrowserClient::with_config(config).expect("Chrome/Chromium not installed");

    // This should timeout on a slow-loading page
    let result = browser.get_html_wait_for(
        "https://httpbin.org/delay/10",
        "body",
        Some(Duration::from_millis(100)),
    );

    // Should fail due to timeout
    assert!(result.is_err(), "Should timeout on slow page");
}

#[test]
#[ignore] // Requires Chrome/Chromium
fn test_image_loading_disabled() {
    let config = BrowserConfig {
        headless: true,
        window_width: 1920,
        window_height: 1080,
        timeout: Duration::from_secs(30),
        disable_images: true,
        user_agent: None,
    };

    let browser = BrowserClient::with_config(config).expect("Chrome/Chromium not installed");

    // Just verify it works with images disabled
    let result = browser.get_html("https://example.com");
    assert!(result.is_ok(), "Should work with images disabled");
}

#[test]
#[ignore] // Requires Chrome/Chromium
fn test_custom_user_agent() {
    let custom_ua = "CustomTestBot/1.0";

    let config = BrowserConfig {
        headless: true,
        window_width: 1920,
        window_height: 1080,
        timeout: Duration::from_secs(30),
        disable_images: true,
        user_agent: Some(custom_ua.to_string()),
    };

    let browser = BrowserClient::with_config(config).expect("Chrome/Chromium not installed");

    let result = browser.execute_script("https://example.com", "navigator.userAgent");

    assert!(result.is_ok(), "Failed to get user agent");

    let ua = result.unwrap();
    assert!(ua.contains("CustomTestBot"), "Should use custom user agent");
}
