use rust_manga_scraper::browser::{BrowserConfig, BrowserManager};
use rust_manga_scraper::sources::nyxscans_browser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing NyxScans with Browser ===");
    println!();

    let config = BrowserConfig::default();
    let manager = BrowserManager::new(config)?;
    println!("Browser manager created");
    println!();

    // Test search
    println!("=== Testing Search ===");
    match nyxscans_browser::search_manga_with_urls_browser(&manager, "") {
        Ok(results) => {
            println!("  Found {} manga", results.len());
            if results.len() > 0 {
                println!();
                println!("  First 3 manga:");
                for (i, (manga, url)) in results.iter().take(3).enumerate() {
                    println!("    {}. {} - {}", i + 1, manga.title, url);
                }

                // Test getting chapters for first manga
                if let Some((_, url)) = results.first() {
                    println!();
                    println!("=== Testing Chapters for First Manga ===");
                    match nyxscans_browser::get_chapters_browser(&manager, url) {
                        Ok(chapters) => {
                            println!("  Found {} chapters", chapters.len());
                            if chapters.len() > 0 {
                                println!("  First few chapters:");
                                for (i, chapter) in chapters.iter().take(5).enumerate() {
                                    println!("    {}. {}", i + 1, chapter.chapter_number);
                                }
                            }
                        }
                        Err(e) => println!("  Chapter error: {}", e),
                    }
                }
            }
        }
        Err(e) => println!("  Error: {}", e),
    }

    println!();
    println!("Test complete!");
    Ok(())
}
