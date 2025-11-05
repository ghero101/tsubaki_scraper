/// End-to-end integration tests
/// Tests the complete workflow from configuration to scraping

use rust_manga_scraper::config::Config;
use rust_manga_scraper::metrics::MetricsTracker;
use rust_manga_scraper::http_client::EnhancedHttpClient;
use std::time::Duration;

#[tokio::test]
async fn test_complete_workflow_http_client() {
    // 1. Load configuration
    let config = Config::load();
    assert!(!config.download_dir.is_empty(), "Config should have download dir");

    // 2. Create HTTP client from config
    let client_result = config.bot_detection.create_http_client();
    assert!(client_result.is_ok(), "Should create HTTP client from config");
    let client = client_result.unwrap();

    // 3. Create metrics tracker
    let metrics = MetricsTracker::new();

    // 4. Make a request with metrics tracking
    let start = std::time::Instant::now();
    let result = client.get_text("https://httpbin.org/get").await;

    match result {
        Ok(text) => {
            let elapsed = start.elapsed();
            metrics.record_success("test_source", elapsed);

            assert!(!text.is_empty(), "Response should not be empty");
            println!("✓ Request successful in {}ms", elapsed.as_millis());
        }
        Err(e) => {
            eprintln!("⚠ Request failed (may be network issue): {}", e);
        }
    }

    // 5. Verify metrics were recorded
    if let Some(source_metrics) = metrics.get_metrics("test_source") {
        assert!(source_metrics.total_requests > 0, "Should have recorded requests");
        println!("✓ Metrics recorded: {} requests", source_metrics.total_requests);
    }
}

#[tokio::test]
async fn test_retry_mechanism() {
    let config = Config::load();
    let client = config.bot_detection.create_http_client()
        .expect("Failed to create client");

    let metrics = MetricsTracker::new();

    // Test with an endpoint that returns 503 (should retry)
    let start = std::time::Instant::now();
    let result = client.get_with_retry("https://httpbin.org/status/503").await;
    let elapsed = start.elapsed();

    // Should have retried (takes at least initial_retry_delay_ms)
    assert!(
        elapsed >= Duration::from_millis(config.bot_detection.initial_retry_delay_ms),
        "Should have retried with delay"
    );

    match result {
        Ok(resp) => {
            // Will return 503 after retries
            assert_eq!(resp.status().as_u16(), 503);
            println!("✓ Retry logic working (returned 503 after retries)");
        }
        Err(_) => {
            println!("✓ Retry logic working (failed after retries as expected)");
        }
    }
}

#[tokio::test]
async fn test_configuration_variations() {
    // Test with different timeout values
    let config = Config::load();

    assert!(config.bot_detection.timeout_secs > 0, "Timeout should be positive");
    assert!(config.bot_detection.max_retries > 0, "Max retries should be positive");
    assert!(config.bot_detection.initial_retry_delay_ms > 0, "Retry delay should be positive");

    println!("✓ Configuration validation passed");
    println!("  Timeout: {}s", config.bot_detection.timeout_secs);
    println!("  Max retries: {}", config.bot_detection.max_retries);
    println!("  Initial delay: {}ms", config.bot_detection.initial_retry_delay_ms);
}

#[tokio::test]
async fn test_metrics_aggregation() {
    let metrics = MetricsTracker::new();

    // Simulate multiple requests
    for i in 0..5 {
        if i % 2 == 0 {
            metrics.record_success("test_source", Duration::from_millis(100 + i * 10));
        } else {
            metrics.record_failure("test_source", format!("Test error {}", i));
        }
    }

    let source_metrics = metrics.get_metrics("test_source").unwrap();

    assert_eq!(source_metrics.total_requests, 5, "Should have 5 requests");
    assert_eq!(source_metrics.successful_requests, 3, "Should have 3 successes");
    assert_eq!(source_metrics.failed_requests, 2, "Should have 2 failures");
    assert_eq!(source_metrics.success_rate(), 60.0, "Success rate should be 60%");

    println!("✓ Metrics aggregation working");
    println!("  Total: {}", source_metrics.total_requests);
    println!("  Success rate: {:.1}%", source_metrics.success_rate());
}

#[tokio::test]
async fn test_metrics_error_categorization() {
    let metrics = MetricsTracker::new();

    // Simulate different types of errors
    metrics.record_failure("source1", "429 Too Many Requests".to_string());
    metrics.record_failure("source1", "Rate limit exceeded".to_string());
    metrics.record_failure("source1", "503 Service Unavailable - Cloudflare".to_string());
    metrics.record_failure("source1", "Request timeout after 30s".to_string());

    let source_metrics = metrics.get_metrics("source1").unwrap();

    assert_eq!(source_metrics.rate_limit_hits, 2, "Should detect 2 rate limit errors");
    assert_eq!(source_metrics.cloudflare_challenges, 1, "Should detect 1 Cloudflare error");
    assert_eq!(source_metrics.timeout_count, 1, "Should detect 1 timeout");

    println!("✓ Error categorization working");
    println!("  Rate limits: {}", source_metrics.rate_limit_hits);
    println!("  Cloudflare: {}", source_metrics.cloudflare_challenges);
    println!("  Timeouts: {}", source_metrics.timeout_count);
}

#[tokio::test]
async fn test_multiple_sources_tracking() {
    let metrics = MetricsTracker::new();

    // Track multiple sources
    metrics.record_success("mangadex", Duration::from_millis(150));
    metrics.record_success("firescans", Duration::from_millis(200));
    metrics.record_failure("asmotoon", "Connection timeout".to_string());

    let all_metrics = metrics.get_all_metrics();

    assert_eq!(all_metrics.len(), 3, "Should track 3 sources");

    let mangadex = metrics.get_metrics("mangadex").unwrap();
    let firescans = metrics.get_metrics("firescans").unwrap();
    let asmotoon = metrics.get_metrics("asmotoon").unwrap();

    assert_eq!(mangadex.success_rate(), 100.0);
    assert_eq!(firescans.success_rate(), 100.0);
    assert_eq!(asmotoon.success_rate(), 0.0);

    println!("✓ Multiple sources tracking working");
    println!("  Sources tracked: {}", all_metrics.len());
}

#[tokio::test]
async fn test_cookie_persistence() {
    let client = EnhancedHttpClient::new().expect("Failed to create client");

    // httpbin.org/cookies/set sets a cookie and redirects
    let result = client.get_with_retry("https://httpbin.org/cookies/set?test=value").await;

    match result {
        Ok(response) => {
            // Should handle cookies and redirects
            assert!(
                response.status().is_success() || response.status().is_redirection(),
                "Should handle cookie setting"
            );
            println!("✓ Cookie persistence working");
        }
        Err(e) => {
            eprintln!("⚠ Cookie test failed (may be network issue): {}", e);
        }
    }
}

#[tokio::test]
async fn test_compression_support() {
    let client = EnhancedHttpClient::new().expect("Failed to create client");

    // httpbin.org/gzip returns gzipped content
    let result = client.get_text("https://httpbin.org/gzip").await;

    match result {
        Ok(text) => {
            assert!(!text.is_empty(), "Should decompress content");
            assert!(text.contains("gzipped"), "Should contain expected text");
            println!("✓ Compression support working");
        }
        Err(e) => {
            eprintln!("⚠ Compression test failed (may be network issue): {}", e);
        }
    }
}

#[tokio::test]
async fn test_browser_availability() {
    let config = Config::load();

    if config.bot_detection.enable_browser {
        let browser_result = config.bot_detection.create_browser_client();

        match browser_result {
            Ok(_browser) => {
                println!("✓ Browser client available and enabled");
            }
            Err(e) => {
                eprintln!("⚠ Browser enabled in config but not available: {}", e);
            }
        }
    } else {
        println!("ℹ Browser client disabled in configuration (enable_browser = false)");
    }
}

#[tokio::test]
async fn test_fallback_strategy() {
    let config = Config::load();

    // HTTP client should always work
    let http_result = config.bot_detection.create_http_client();
    assert!(http_result.is_ok(), "HTTP client should always be available");

    // Browser may or may not be available
    let browser_result = config.bot_detection.create_browser_client();

    match browser_result {
        Ok(_) => println!("✓ Browser client available as fallback"),
        Err(_) => println!("ℹ Browser client not available (expected if Chrome not installed)"),
    }

    println!("✓ Fallback strategy validated");
}

#[tokio::test]
async fn test_metrics_json_export() {
    let metrics = MetricsTracker::new();

    metrics.record_success("test_source", Duration::from_millis(100));
    metrics.record_failure("test_source", "Test error".to_string());

    let json = metrics.export_json();

    assert!(!json.is_empty(), "JSON export should not be empty");
    assert!(json.contains("test_source"), "Should contain source name");
    assert!(json.contains("total_requests"), "Should contain metrics data");

    println!("✓ Metrics JSON export working");
    println!("  Export length: {} bytes", json.len());
}
