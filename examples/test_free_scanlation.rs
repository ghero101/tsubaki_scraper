use rust_manga_scraper::browser::{BrowserConfig, BrowserManager};
use rust_manga_scraper::sources::{daycomics_browser, lunatoons_browser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing Free Scanlation Sites ===");
    println!();

    let config = BrowserConfig::default();
    let manager = BrowserManager::new(config)?;

    // Test DayComics
    println!("=== Testing DayComics ===");
    match daycomics_browser::search_manga_with_urls_browser(&manager, "") {
        Ok(results) => {
            println!("  Found {} manga", results.len());
            if results.len() > 0 {
                if let Some((manga, url)) = results.first() {
                    println!("  First: {} - {}", manga.title, url);
                    match daycomics_browser::get_chapters_browser(&manager, url) {
                        Ok(chapters) => println!("    Chapters: {}", chapters.len()),
                        Err(e) => println!("    Chapter error: {}", e),
                    }
                }
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    // Test LunaToons
    println!();
    println!("=== Testing LunaToons ===");
    match lunatoons_browser::search_manga_with_urls_browser(&manager, "") {
        Ok(results) => {
            println!("  Found {} manga", results.len());
            if results.len() > 0 {
                if let Some((manga, url)) = results.first() {
                    println!("  First: {} - {}", manga.title, url);
                    match lunatoons_browser::get_chapters_browser(&manager, url) {
                        Ok(chapters) => println!("    Chapters: {}", chapters.len()),
                        Err(e) => println!("    Chapter error: {}", e),
                    }
                }
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!();
    println!("=== Test Complete ===");
    Ok(())
}
