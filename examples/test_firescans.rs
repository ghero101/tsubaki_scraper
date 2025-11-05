use reqwest::Client;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .build()?;

    println!("Testing FireScans...\n");

    // Get manga
    let manga_list = rust_manga_scraper::sources::firescans::search_manga_with_urls(&client, "").await?;
    println!("✓ Found {} manga", manga_list.len());

    if let Some((manga, url)) = manga_list.first() {
        println!("Testing first manga: {}", manga.title);
        println!("URL: {}", url);

        // Try to get chapters
        let chapters = rust_manga_scraper::sources::firescans::get_chapters(&client, url).await?;
        println!("✓ Found {} chapters", chapters.len());

        if chapters.is_empty() {
            println!("\n⚠️ No chapters found - this is the issue!");

            // Let's fetch the page and see what's there
            println!("\nFetching page to inspect...");
            let response = client.get(url).send().await?.text().await?;

            // Look for chapter-related elements
            println!("\nLooking for chapter indicators in HTML:");
            if response.contains("chapter-list") {
                println!("  ✓ Found 'chapter-list' in HTML");
            }
            if response.contains("wp-manga-chapter") {
                println!("  ✓ Found 'wp-manga-chapter' in HTML");
            }
            if response.contains("eplister") {
                println!("  ✓ Found 'eplister' (episode lister) in HTML");
            }
            if response.contains("chapterlist") {
                println!("  ✓ Found 'chapterlist' in HTML");
            }
            if response.contains("manga_get_chapters") {
                println!("  ✓ Found 'manga_get_chapters' AJAX endpoint in HTML");
            }
            if response.contains("data-id") {
                println!("  ✓ Found 'data-id' attribute in HTML");
            }

            // Save a snippet to file for inspection
            std::fs::write("/tmp/firescans_page.html", &response)?;
            println!("\n✓ Saved full page to /tmp/firescans_page.html for inspection");
        } else {
            println!("\nFirst 3 chapters:");
            for (i, ch) in chapters.iter().take(3).enumerate() {
                println!("  {}. {} - {}", i + 1, ch.chapter_number, ch.url);
            }
        }
    }

    Ok(())
}
