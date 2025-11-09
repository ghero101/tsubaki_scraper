use rust_manga_scraper::browser::{BrowserConfig, BrowserManager};
use rust_manga_scraper::sources::hivetoons_browser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing HiveToons with Browser Automation ===\n");

    // Create browser manager
    println!("1. Creating browser manager...");
    let config = BrowserConfig::default();
    let manager = BrowserManager::new(config)?;
    println!("   ✓ Browser manager created\n");

    // Test manga search
    println!("2. Searching for manga (fetching series list)...");
    let results = hivetoons_browser::search_manga_with_urls_browser(&manager, "")?;
    println!("   ✓ Found {} manga\n", results.len());

    if results.is_empty() {
        println!("   ⚠ No manga found - site may have changed structure");
        return Ok(());
    }

    // Test first 3 manga for chapters
    println!("3. Testing chapter extraction for first 3 manga:\n");
    for (manga, url) in results.iter().take(3) {
        println!("   Testing: {}", manga.title);
        println!("   URL: {}", url);

        match hivetoons_browser::get_chapters_browser(&manager, url) {
            Ok(chapters) => {
                println!("   ✓ Chapters: {}", chapters.len());
                if !chapters.is_empty() {
                    println!("      First: {}", chapters.first().unwrap().chapter_number);
                    println!("      Last: {}", chapters.last().unwrap().chapter_number);
                }
            }
            Err(e) => {
                println!("   ✗ Error: {}", e);
            }
        }
        println!();
    }

    // Calculate total
    println!("4. Calculating total chapters...");
    let mut total_chapters = 0;
    for (_, url) in results.iter().take(3) {
        if let Ok(chapters) = hivetoons_browser::get_chapters_browser(&manager, url) {
            total_chapters += chapters.len();
        }
    }

    println!("\n============================================");
    println!("RESULTS:");
    println!("  Manga found: {}", results.len());
    println!("  Total chapters (from 3 manga): {}", total_chapters);
    println!("============================================");

    if total_chapters > 3 {
        println!(
            "\n✅ SUCCESS! Found {} chapters (was 3 with HTTP scraping)",
            total_chapters
        );
        println!("Browser automation is working for HiveToons!");
    } else {
        println!("\n⚠ WARNING: Only found {} chapters", total_chapters);
        println!("May need to adjust selectors or wait times");
    }

    Ok(())
}
