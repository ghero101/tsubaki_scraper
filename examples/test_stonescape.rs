use reqwest::Client;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .build()?;

    println!("Testing StoneScape...\n");

    // Get manga
    let manga_list = rust_manga_scraper::sources::stonescape::search_manga_with_urls(&client, "").await?;
    println!("✓ Found {} manga", manga_list.len());

    if let Some((manga, url)) = manga_list.first() {
        println!("Testing first manga: {}", manga.title);
        println!("URL: {}", url);

        // Try to get chapters
        let chapters = rust_manga_scraper::sources::stonescape::get_chapters(&client, url).await?;
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
            if response.contains("btn-read-first") || response.contains("btn-read-last") {
                println!("  ✓ Found 'btn-read-first/last' buttons in HTML");
            }
            if response.contains("manga-chapters-holder") {
                println!("  ✓ Found 'manga-chapters-holder' in HTML");
            }

            // Look for chapter links
            let chapter_links: Vec<_> = response
                .lines()
                .filter(|line| line.contains("chapter-") && line.contains("href"))
                .take(5)
                .collect();

            if !chapter_links.is_empty() {
                println!("\nSample chapter link patterns:");
                for link in chapter_links {
                    println!("  {}", link.trim());
                }
            }

            // Save a snippet to file for inspection
            std::fs::write("/tmp/stonescape_page.html", &response)?;
            println!("\n✓ Saved full page to /tmp/stonescape_page.html for inspection");
        } else {
            println!("\nFirst 3 chapters:");
            for (i, ch) in chapters.iter().take(3).enumerate() {
                println!("  {}. {} - {}", i + 1, ch.chapter_number, ch.url);
            }
        }
    }

    Ok(())
}
