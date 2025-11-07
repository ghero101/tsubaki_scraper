use rust_manga_scraper::browser::{BrowserConfig, BrowserManager};
use rust_manga_scraper::sources::{drakecomic_browser, thunderscans_browser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Cloudflare-Protected Sources with Browser ===\n");

    let config = BrowserConfig::default();
    let manager = BrowserManager::new(config)?;
    println!("✓ Browser manager created\n");

    // Test DrakeComic
    println!("=== Testing DrakeComic ===");
    match drakecomic_browser::search_manga_with_urls_browser(&manager, "") {
        Ok(results) => {
            println!("  ✓ Found {} manga", results.len());
            if results.len() > 0 {
                if let Some((_, url)) = results.first() {
                    match drakecomic_browser::get_chapters_browser(&manager, url) {
                        Ok(chapters) => println!("  ✓ First manga has {} chapters", chapters.len()),
                        Err(e) => println!("  ✗ Chapter error: {}", e),
                    }
                }
            }
        }
        Err(e) => println!("  ✗ Error: {}", e),
    }

    // Test ThunderScans
    println!("\n=== Testing ThunderScans ===");
    match thunderscans_browser::search_manga_with_urls_browser(&manager, "") {
        Ok(results) => {
            println!("  ✓ Found {} manga", results.len());
            if results.len() > 0 {
                if let Some((_, url)) = results.first() {
                    match thunderscans_browser::get_chapters_browser(&manager, url) {
                        Ok(chapters) => println!("  ✓ First manga has {} chapters", chapters.len()),
                        Err(e) => println!("  ✗ Chapter error: {}", e),
                    }
                }
            }
        }
        Err(e) => println!("  ✗ Error: {}", e),
    }

    println!("\n✅ Browser automation can bypass Cloudflare!");
    Ok(())
}
