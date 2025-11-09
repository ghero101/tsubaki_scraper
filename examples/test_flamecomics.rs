use reqwest::Client;
use rust_manga_scraper::sources::flamecomics;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    println!("=== Testing FlameComics ===\n");

    // Test manga search
    let results = flamecomics::search_manga_with_urls(&client, "").await?;
    println!("✓ Found {} manga\n", results.len());

    // Test first 3 manga for chapters
    for (manga, url) in results.iter().take(3) {
        println!("Testing: {}", manga.title);
        println!("  URL: {}", url);

        match flamecomics::get_chapters(&client, &url).await {
            Ok(chapters) => {
                println!("  ✓ Chapters: {}", chapters.len());
                if !chapters.is_empty() {
                    println!("    First: {}", chapters.first().unwrap().chapter_number);
                    println!("    Last: {}", chapters.last().unwrap().chapter_number);
                }
            }
            Err(e) => {
                println!("  ✗ Error: {}", e);
            }
        }
        println!();
    }

    // Calculate total from already tested manga
    let mut total_chapters = 0;
    for (_, url) in results.iter().take(3) {
        if let Ok(chapters) = flamecomics::get_chapters(&client, url).await {
            total_chapters += chapters.len();
        }
    }

    println!("============================================");
    println!(
        "TOTAL CHAPTERS FROM {} MANGA: {}",
        results.len().min(3),
        total_chapters
    );
    println!("============================================");

    Ok(())
}
