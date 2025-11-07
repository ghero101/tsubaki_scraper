use rust_manga_scraper::browser::{BrowserConfig, BrowserManager};
use rust_manga_scraper::sources::qiscans_browser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Debugging QIScans ===\n");

    let config = BrowserConfig::default();
    let manager = BrowserManager::new(config)?;

    println!("Calling search_manga_with_urls_browser...");
    match qiscans_browser::search_manga_with_urls_browser(&manager, "") {
        Ok(results) => {
            println!("✓ Found {} manga", results.len());
            if results.is_empty() {
                println!("⚠ No manga found - check qiscans_series_list.html for debugging");
            } else {
                for (i, (manga, url)) in results.iter().enumerate() {
                    println!("  {}. {} - {}", i + 1, manga.title, url);
                }
            }
        }
        Err(e) => println!("✗ Error: {}", e),
    }

    println!("\n✅ Check qiscans_series_list.html to debug selectors");
    Ok(())
}
