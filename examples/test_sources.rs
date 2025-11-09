use reqwest::Client;
/// Test script for problematic manga sources
///
/// Usage:
///   cargo run --example test_sources
///   cargo run --example test_sources -- --browser  # Test browser sources
use std::time::Instant;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args: Vec<String> = std::env::args().collect();
    let test_browser = args.contains(&"--browser".to_string());

    println!("\n╔═══════════════════════════════════════════════════════╗");
    println!("║     Manga Scraper Source Test Suite                  ║");
    println!("╚═══════════════════════════════════════════════════════╝\n");

    // Test HTTP client
    println!("🔍 Testing Enhanced HTTP Client...\n");
    test_http_client().await;

    // Test configuration
    println!("\n⚙️  Testing Configuration System...\n");
    test_configuration().await;

    // Test metrics
    println!("\n📊 Testing Metrics System...\n");
    test_metrics().await;

    // Test problematic sources with enhanced HTTP
    println!("\n🌐 Testing Problematic Sources (Enhanced HTTP)...\n");
    test_problematic_sources_http().await;

    // Test browser client if requested
    if test_browser {
        println!("\n🤖 Testing Browser Client...\n");
        test_browser_client().await;

        println!("\n🌐 Testing Browser-Based Sources...\n");
        test_browser_sources().await;
    } else {
        println!("\n⚠️  Skipping browser tests (use --browser flag to enable)");
    }

    println!("\n✅ Test suite completed!\n");
}

async fn test_http_client() {
    use rust_manga_scraper::http_client::EnhancedHttpClient;

    print!("  Creating enhanced HTTP client... ");
    match EnhancedHttpClient::new() {
        Ok(_) => println!("✓ Success"),
        Err(e) => println!("✗ Failed: {}", e),
    }

    print!("  Testing request with retry logic... ");
    match EnhancedHttpClient::new() {
        Ok(client) => {
            let start = Instant::now();
            match client.get_text("https://httpbin.org/delay/1").await {
                Ok(_) => {
                    let elapsed = start.elapsed();
                    println!("✓ Success ({}ms)", elapsed.as_millis());
                }
                Err(e) => println!("✗ Failed: {}", e),
            }
        }
        Err(e) => println!("✗ Failed to create client: {}", e),
    }

    print!("  Testing retry on 503 error... ");
    match EnhancedHttpClient::new() {
        Ok(client) => {
            let start = Instant::now();
            let _ = client
                .get_with_retry("https://httpbin.org/status/503")
                .await;
            let elapsed = start.elapsed();
            if elapsed.as_millis() >= 500 {
                println!("✓ Retried as expected ({}ms)", elapsed.as_millis());
            } else {
                println!("⚠ May not have retried ({}ms)", elapsed.as_millis());
            }
        }
        Err(e) => println!("✗ Failed: {}", e),
    }
}

async fn test_configuration() {
    use rust_manga_scraper::config::Config;

    print!("  Loading configuration... ");
    let config = Config::load();
    println!("✓ Loaded");

    println!("    Download dir: {}", config.download_dir);
    println!(
        "    Enhanced client enabled: {}",
        config.bot_detection.enable_enhanced_client
    );
    println!(
        "    Browser enabled: {}",
        config.bot_detection.enable_browser
    );
    println!("    Max retries: {}", config.bot_detection.max_retries);
    println!("    Timeout: {}s", config.bot_detection.timeout_secs);

    print!("  Creating HTTP client from config... ");
    match config.bot_detection.create_http_client() {
        Ok(_) => println!("✓ Success"),
        Err(e) => println!("✗ Failed: {}", e),
    }

    print!("  Creating browser client from config... ");
    match config.bot_detection.create_browser_client() {
        Ok(_) => println!("✓ Success (Chrome available)"),
        Err(e) => println!("⚠ Not available: {}", e),
    }
}

async fn test_metrics() {
    use rust_manga_scraper::metrics::{track_request, MetricsTracker};
    use std::time::Duration;

    let tracker = MetricsTracker::new();

    print!("  Recording successful request... ");
    let _ = track_request(&tracker, "test_source", async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok::<_, String>(())
    })
    .await;
    println!("✓ Recorded");

    print!("  Recording failed request... ");
    let _ = track_request(&tracker, "test_source", async {
        Err::<(), _>("Test error".to_string())
    })
    .await;
    println!("✓ Recorded");

    if let Some(metrics) = tracker.get_metrics("test_source") {
        println!("    Total requests: {}", metrics.total_requests);
        println!("    Success rate: {:.1}%", metrics.success_rate());
        println!(
            "    Avg response time: {:.2}ms",
            metrics.average_response_time_ms
        );
    }
}

async fn test_problematic_sources_http() {
    use rust_manga_scraper::http_client::EnhancedHttpClient;

    let sources = vec![
        ("MangaDex API", "https://api.mangadex.org/manga?limit=1"),
        ("Asmotoon", "https://asmotoon.com"),
        ("DrakeComic", "https://drakecomic.org"),
        ("Hivetoons", "https://hivetoons.com"),
        ("KenScans", "https://kenscans.com"),
        ("QiScans", "https://qiscans.org"),
        ("NyxScans", "https://nyxscans.com"),
    ];

    let client = EnhancedHttpClient::new().expect("Failed to create client");

    for (name, url) in sources {
        print!("  Testing {:<20} ", name);
        let start = Instant::now();

        match client.get_with_retry(url).await {
            Ok(response) => {
                let status = response.status();
                let elapsed = start.elapsed();

                if status.is_success() {
                    println!("✓ Success ({}ms, status: {})", elapsed.as_millis(), status);
                } else if status.as_u16() == 403 || status.as_u16() == 503 {
                    println!(
                        "⚠ Blocked ({}ms, status: {}) - May need browser",
                        elapsed.as_millis(),
                        status
                    );
                } else {
                    println!("⚠ Status {} ({}ms)", status, elapsed.as_millis());
                }
            }
            Err(e) => {
                let elapsed = start.elapsed();
                if e.is_timeout() {
                    println!("⚠ Timeout ({}ms) - May need browser", elapsed.as_millis());
                } else if e.is_connect() {
                    println!("✗ Connection error - Domain may be down");
                } else {
                    println!("✗ Error: {}", e);
                }
            }
        }
    }
}

async fn test_browser_client() {
    use rust_manga_scraper::browser_client::BrowserClient;

    print!("  Creating browser client... ");
    match BrowserClient::new() {
        Ok(_) => println!("✓ Chrome/Chromium available"),
        Err(e) => {
            println!("✗ Not available: {}", e);
            println!("\n  To install Chrome/Chromium:");
            println!("    Ubuntu/Debian: sudo apt-get install chromium-browser");
            println!("    macOS: brew install chromium");
            println!("    Arch: sudo pacman -S chromium\n");
            return;
        }
    }

    print!("  Testing basic navigation... ");
    match BrowserClient::new() {
        Ok(browser) => {
            let start = Instant::now();
            match browser.get_html("https://example.com") {
                Ok(html) => {
                    let elapsed = start.elapsed();
                    if html.contains("Example Domain") {
                        println!("✓ Success ({}ms)", elapsed.as_millis());
                    } else {
                        println!("⚠ Loaded but unexpected content");
                    }
                }
                Err(e) => println!("✗ Failed: {}", e),
            }
        }
        Err(e) => println!("✗ Failed: {}", e),
    }

    print!("  Testing JavaScript execution... ");
    match BrowserClient::new() {
        Ok(browser) => match browser.execute_script("https://example.com", "document.title") {
            Ok(result) => println!("✓ Success: {}", result),
            Err(e) => println!("✗ Failed: {}", e),
        },
        Err(e) => println!("✗ Failed: {}", e),
    }
}

async fn test_browser_sources() {
    use rust_manga_scraper::sources_browser;

    println!("  Testing Asmotoon (browser)...");
    print!("    Searching for 'one piece'... ");
    match sources_browser::asmotoon_browser::search_manga_with_urls("one piece").await {
        Ok(results) => {
            println!("✓ Found {} manga", results.len());
            if !results.is_empty() {
                println!("      Example: {}", results[0].0.title);
            }
        }
        Err(e) => println!("✗ Failed: {}", e),
    }

    println!("\n  Testing DrakeComic (browser with Cloudflare bypass)...");
    print!("    Fetching manga list... ");
    match sources_browser::drakecomic_browser::search_manga_with_urls().await {
        Ok(results) => {
            println!("✓ Found {} manga (bypassed Cloudflare)", results.len());
            if !results.is_empty() {
                println!("      Example: {}", results[0].0.title);
            }
        }
        Err(e) => println!("✗ Failed: {}", e),
    }

    println!("\n  Testing Hivetoons (browser)...");
    print!("    Fetching manga list... ");
    match sources_browser::hivetoons_browser::search_manga_with_urls().await {
        Ok(results) => {
            println!("✓ Found {} manga", results.len());
        }
        Err(e) => println!("✗ Failed: {}", e),
    }
}
