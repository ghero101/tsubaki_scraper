use reqwest::Client;
use std::time::Duration;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    println!("=== Testing FlameComics New Implementation ===\n");

    // Test search
    match rust_manga_scraper::sources::flamecomics::search_manga_with_urls(&client, "").await {
        Ok(results) => {
            println!("✓ Found {} manga", results.len());
            
            // Test get_chapters on first 3 manga
            let mut total_chapters = 0;
            for (manga, url) in results.iter().take(3) {
                println!("\nTesting: {}", manga.title);
                println!("  URL: {}", url);
                
                match rust_manga_scraper::sources::flamecomics::get_chapters(&client, url).await {
                    Ok(chapters) => {
                        println!("  ✓ Chapters: {}", chapters.len());
                        total_chapters += chapters.len();
                    }
                    Err(e) => println!("  ✗ Error: {}", e),
                }
            }
            
            println!("\n=== TOTAL CHAPTERS: {} ===", total_chapters);
        }
        Err(e) => println!("✗ Search error: {}", e),
    }
}
