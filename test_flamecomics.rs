use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    println!("Testing FlameComics scraper...\n");

    // Test search
    println!("=== Testing search_manga_with_urls ===");
    match rust_manga_scraper::sources::flamecomics::search_manga_with_urls(&client, "").await {
        Ok(results) => {
            println!("✓ Found {} manga", results.len());
            for (manga, url) in results.iter().take(3) {
                println!("  - {} ({})", manga.title, url);
            }

            // Test get_chapters on first manga
            if let Some((manga, url)) = results.first() {
                println!("\n=== Testing get_chapters for: {} ===", manga.title);
                match rust_manga_scraper::sources::flamecomics::get_chapters(&client, url).await {
                    Ok(chapters) => {
                        println!("✓ Found {} chapters", chapters.len());
                        if !chapters.is_empty() {
                            println!("  First: {}", chapters[0].chapter_number);
                            println!("  Last: {}", chapters[chapters.len()-1].chapter_number);
                        }
                    }
                    Err(e) => println!("✗ Chapter fetch error: {}", e),
                }
            }
        }
        Err(e) => println!("✗ Search error: {}", e),
    }
}
