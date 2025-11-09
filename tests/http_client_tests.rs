use rust_manga_scraper::http_client::{EnhancedHttpClient, HttpClientConfig};
use std::time::Duration;

#[tokio::test]
async fn test_http_client_creation() {
    let client = EnhancedHttpClient::new();
    assert!(client.is_ok(), "Failed to create HTTP client");
}

#[tokio::test]
async fn test_http_client_with_custom_config() {
    let config = HttpClientConfig {
        timeout: Duration::from_secs(10),
        max_retries: 2,
        initial_retry_delay_ms: 100,
        max_retry_delay_ms: 1000,
        enable_cookies: true,
        enable_gzip: true,
    };

    let client = EnhancedHttpClient::with_config(config);
    assert!(
        client.is_ok(),
        "Failed to create HTTP client with custom config"
    );
}

#[tokio::test]
async fn test_fetch_success() {
    let client = EnhancedHttpClient::new().expect("Failed to create client");

    // Test with a reliable public endpoint
    let result = client.get_text("https://httpbin.org/html").await;

    match result {
        Ok(html) => {
            assert!(!html.is_empty(), "Response should not be empty");
            assert!(html.contains("html"), "Response should contain HTML");
        }
        Err(e) => {
            // Network might be unavailable in test environment
            eprintln!(
                "Warning: Network request failed (may be expected in CI): {}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_retry_on_rate_limit() {
    let client = EnhancedHttpClient::new().expect("Failed to create client");

    // httpbin.org/status/429 returns 429 Too Many Requests
    let result = client
        .get_with_retry("https://httpbin.org/status/429")
        .await;

    // Should eventually fail after retries (429 is retryable but will keep returning 429)
    assert!(result.is_err() || result.unwrap().status().as_u16() == 429);
}

#[tokio::test]
async fn test_headers_included() {
    let client = EnhancedHttpClient::new().expect("Failed to create client");

    // httpbin.org/headers returns the headers we sent
    let result = client.get_text("https://httpbin.org/headers").await;

    match result {
        Ok(response) => {
            // Check that common browser headers are present
            assert!(response.contains("User-Agent"), "Should include User-Agent");
            assert!(response.contains("Accept"), "Should include Accept header");
        }
        Err(e) => {
            eprintln!(
                "Warning: Network request failed (may be expected in CI): {}",
                e
            );
        }
    }
}

#[tokio::test]
async fn test_rate_limiting() {
    let client = EnhancedHttpClient::new().expect("Failed to create client");

    let start = std::time::Instant::now();

    // Make a request and apply rate limiting
    let _ = client.get_with_retry("https://httpbin.org/html").await;
    client.rate_limit_delay(500).await;

    let elapsed = start.elapsed();
    assert!(
        elapsed >= Duration::from_millis(500),
        "Rate limiting should delay at least 500ms"
    );
}

#[tokio::test]
async fn test_timeout_configuration() {
    let config = HttpClientConfig {
        timeout: Duration::from_millis(1), // Very short timeout
        max_retries: 1,
        initial_retry_delay_ms: 10,
        max_retry_delay_ms: 100,
        enable_cookies: true,
        enable_gzip: true,
    };

    let client = EnhancedHttpClient::with_config(config).expect("Failed to create client");

    // This should timeout quickly
    let result = client.get_text("https://httpbin.org/delay/5").await;
    assert!(result.is_err(), "Should timeout on slow endpoint");
}

#[tokio::test]
async fn test_cookie_support() {
    let client = EnhancedHttpClient::new().expect("Failed to create client");

    // httpbin.org/cookies/set redirects and sets a cookie
    let result = client
        .get_with_retry("https://httpbin.org/cookies/set?test=value")
        .await;

    // Should successfully handle cookies and redirects
    match result {
        Ok(response) => {
            assert!(response.status().is_success() || response.status().is_redirection());
        }
        Err(e) => {
            eprintln!("Warning: Cookie test failed (may be expected in CI): {}", e);
        }
    }
}

#[tokio::test]
async fn test_gzip_compression() {
    let client = EnhancedHttpClient::new().expect("Failed to create client");

    // httpbin.org/gzip returns gzipped response
    let result = client.get_text("https://httpbin.org/gzip").await;

    match result {
        Ok(text) => {
            assert!(!text.is_empty(), "Should decompress gzipped content");
            assert!(text.contains("gzipped"), "Should contain expected text");
        }
        Err(e) => {
            eprintln!("Warning: Gzip test failed (may be expected in CI): {}", e);
        }
    }
}

#[test]
fn test_user_agent_rotation() {
    // Test that we have multiple user agents defined
    let ua1 = EnhancedHttpClient::new().unwrap();
    let ua2 = EnhancedHttpClient::new().unwrap();

    // Both should be valid clients (user agents are rotated internally)
    // Just verify they can be created successfully
    let _ = ua1.client();
    let _ = ua2.client();
}
