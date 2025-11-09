/// Test MangaDex API directly
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36")
        .build()?;

    println!("Testing MangaDex API...");

    // Test the actual function
    match rust_manga_scraper::sources::mangadex::search_all_manga(
        &client,
        rust_manga_scraper::sources::mangadex::BASE_URL,
    )
    .await
    {
        Ok(manga) => {
            println!("✓ MangaDex returned {} manga", manga.len());
            if !manga.is_empty() {
                println!("First manga: {}", manga[0].title);
            }
        }
        Err(e) => {
            println!("✗ MangaDex error: {}", e);
        }
    }

    Ok(())
}
