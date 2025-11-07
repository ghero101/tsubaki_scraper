use rust_manga_scraper::browser::{BrowserConfig, BrowserManager, BrowserScraper};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Browser Module ===\n");

    // Create browser manager with default config
    println!("1. Creating browser manager...");
    let config = BrowserConfig::default();
    let manager = BrowserManager::new(config)?;
    println!("   ✓ Browser manager created\n");

    // Create a new tab
    println!("2. Creating new tab...");
    let tab = manager.new_tab()?;
    println!("   ✓ Tab created\n");

    // Create scraper
    println!("3. Creating scraper...");
    let scraper = BrowserScraper::new(tab);
    println!("   ✓ Scraper created\n");

    // Navigate to example.com
    println!("4. Navigating to https://example.com...");
    scraper.navigate("https://example.com")?;
    println!("   ✓ Navigation complete\n");

    // Wait for h1 element
    println!("5. Waiting for <h1> element...");
    scraper.wait_for_selector("h1")?;
    println!("   ✓ Element found\n");

    // Extract HTML
    println!("6. Extracting HTML...");
    let html = scraper.get_html()?;
    println!("   ✓ Extracted {} bytes of HTML\n", html.len());

    // Verify content
    println!("7. Verifying content...");
    if html.contains("Example Domain") {
        println!("   ✓ Content verified - found 'Example Domain'\n");
    } else {
        println!("   ✗ Content verification failed\n");
        return Err("Expected content not found".into());
    }

    // Test JavaScript evaluation
    println!("8. Testing JavaScript evaluation...");
    let title = scraper.evaluate_script("document.title")?;
    println!("   ✓ Page title: {}\n", title);

    println!("============================================");
    println!("✅ ALL TESTS PASSED!");
    println!("============================================");
    println!("\nBrowser automation module is working correctly!");
    println!("Next: Integrate with HiveToons/KenScans sources");

    Ok(())
}
