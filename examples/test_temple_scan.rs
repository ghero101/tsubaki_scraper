use rust_manga_scraper::browser::{BrowserConfig, BrowserManager};
use rust_manga_scraper::sources::temple_scan_browser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing TempleScan ===");
    println!();

    let config = BrowserConfig::default();
    let manager = BrowserManager::new(config)?;

    match temple_scan_browser::search_manga_with_urls_browser(&manager, "") {
        Ok(results) => {
            println!("Found {} manga", results.len());
            if results.len() > 0 {
                println!("First few:");
                for (i, (manga, url)) in results.iter().take(3).enumerate() {
                    println!("  {}. {} - {}", i + 1, manga.title, url);
                }

                // Test chapters for first manga
                if let Some((_, url)) = results.first() {
                    println!();
                    println!("Testing chapters for first manga...");
                    match temple_scan_browser::get_chapters_browser(&manager, url) {
                        Ok(chapters) => println!("  Chapters: {}", chapters.len()),
                        Err(e) => println!("  Chapter error: {}", e),
                    }
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}
